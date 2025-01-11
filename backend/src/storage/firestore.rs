use crate::{
    api::{email_verification::EmailVerification, measurement::Measurement, user::User},
    helper::{mail::MailHelper, token},
    settings::settings::Settings,
    storage::{
        error::{not_found, undefined, Error, ErrorCode},
        Storage,
    },
};

use async_trait::async_trait;
use chrono::{DateTime, Local, Timelike};
use firestore::{
    errors::{BackoffError, FirestoreError::DataConflictError},
    path, paths, FirestoreDb, FirestoreDbOptions, FirestoreQueryDirection, FirestoreResult,
};
use futures::{stream::BoxStream, FutureExt, TryStreamExt};
use std::sync::Arc;
use tracing::{error, info};

const EMAIL_VERIFICATION_COLLECTION: &'static str = "email_verification";
const MEASUREMENT_COLLECTION: &'static str = "measurement";
const USER_COLLECTION: &'static str = "user";

fn database_error(e: &Error) -> BackoffError<Error> {
    return BackoffError::permanent(e.clone());
}

#[derive(Clone)]
pub struct FirestoreStorage {
    db: FirestoreDb,
}

impl FirestoreStorage {
    pub async fn new(project_id: &str, database_id: &str) -> FirestoreStorage {
        let db = match FirestoreDb::with_options(
            FirestoreDbOptions::new(project_id.to_string())
                .with_database_id(database_id.to_string()),
        )
        .await
        {
            Ok(db) => db,
            Err(err) => panic!(
                "Unable create firestore db for project: {}. Error: {}",
                project_id, err
            ),
        };

        return FirestoreStorage { db };
    }
}

#[async_trait]
impl Storage for FirestoreStorage {
    // The id of the passed measurement is ignored. An id will be assigned automtically
    // We only allow saving at most one
    // Because of limitations of firestore, this method does some funky stuff
    // We use <device-id>-<datetime-to-minute-precision> as document id and we
    // only allow saving at even numbered minutes, so we round the minute down.
    //
    // Examples:
    // device_id: 1234 recorded_at: 2024-12-28T22:08:37.519762489Z document_id: 1234-2024-12-28T22:08
    // device_id: 1234 recorded_at: 2024-12-28T22:09:37.519762489Z document_id: 1234-2024-12-28T22:08
    // (Note how the document_id uses the minute 08 even when recorded_at is at 09)
    //
    // We do this because the firmware sometimes sends duplicated requests a few
    // miliseconds apart by using always rounding down to an even minute we make
    // sure that duplicated requests will always have the same id, so won't be
    // accepted
    async fn save_measurement(&self, measurement: Measurement) -> Result<Measurement, Error> {
        let mut recorded_at = measurement.recorded_at;
        let minute = recorded_at.minute();
        if minute % 2 != 0 {
            recorded_at = recorded_at.with_minute(minute - 1).unwrap();
        }
        let time = recorded_at.format("%Y-%m-%dT%H:%M").to_string();
        let document_id = format!("{}-{}", measurement.device_id, time);
        let inserted: Measurement = match self
            .db
            .fluent()
            .insert()
            .into(MEASUREMENT_COLLECTION)
            .document_id(&document_id)
            .object(&measurement)
            .execute()
            .await
        {
            Ok(inserted) => inserted,
            // If the record already exists, we just return it
            Err(DataConflictError(_)) => {
                info!(
                    "Trying to insert already existing measurement: {}",
                    &document_id
                );

                match self
                    .db
                    .fluent()
                    .select()
                    .by_id_in(MEASUREMENT_COLLECTION)
                    .obj()
                    .one(&document_id)
                    .await
                {
                    Ok(f) => match f {
                        Some(f) => {
                            return Ok(f);
                        }
                        None => {
                            error!(
                                "Couldn't find already existing measurement: {}",
                                &document_id
                            );
                            return Err(Error {
                                code: ErrorCode::UndefinedError,
                            });
                        }
                    },
                    Err(e) => {
                        error!("Error: {}", e);
                        return Err(Error {
                            code: ErrorCode::UndefinedError,
                        });
                    }
                };
            }
            Err(err) => {
                error!("Error: {}", err);
                return Err(Error {
                    code: ErrorCode::UndefinedError,
                });
            }
        };

        Ok(Measurement {
            id: inserted.id,
            ..measurement
        })
    }

    async fn get_measurements(
        &self,
        device_id: String,
        since: DateTime<Local>,
        num_records: u32,
    ) -> Result<Vec<Measurement>, Error> {
        match self
            .db
            .fluent()
            .select()
            .from(MEASUREMENT_COLLECTION)
            .filter(|q| {
                q.for_all([
                    q.field(path!(Measurement::device_id)).eq(device_id.clone()),
                    q.field(path!(Measurement::recorded_at))
                        .less_than_or_equal(since),
                ])
            })
            .order_by([(
                path!(Measurement::recorded_at),
                FirestoreQueryDirection::Descending,
            )])
            .limit(num_records)
            .obj()
            .query()
            .await
        {
            Ok(found) => {
                return Ok(found);
            }
            Err(err) => {
                error!("Error: {}", err);
                return Err(Error {
                    code: ErrorCode::UndefinedError,
                });
            }
        };
    }

    /// Saves a new user to the DB, creates an e-mail verification token and sends
    /// an e-mail for verification. The user needs to verify their e-mail before
    /// they can use their accout
    async fn sign_up_user(
        &self,
        user: User,
        settings: Arc<Settings>,
        mail_helper: Arc<dyn MailHelper>,
    ) -> Result<User, Error> {
        // Even though this says, it's a transaction, firestore doesn't really
        // allow to do this transactionally, so it doesn't really work as a
        // transaction. Leaving it like this in case we change to a storage
        // system that allows transactions
        match self
            .db
            .run_transaction(|db, _| {
                let u = user.clone();
                let s = settings.clone();
                let mh = mail_helper.clone();
                async move {
                    let inserted: User = match db
                        .fluent()
                        .insert()
                        .into(USER_COLLECTION)
                        .document_id(&u.id)
                        .object(&u)
                        .execute()
                        .await
                    {
                        Ok(inserted) => inserted,
                        Err(e) => {
                            error!("Error inserting user. {}", e);
                            return Err::<User, BackoffError<Error>>(database_error(&undefined()));
                        }
                    };

                    let verification_token = token::alphanumeric(100);
                    let verification = EmailVerification {
                        id: inserted.id.clone(),
                        token: verification_token,
                        recorded_at: Local::now(),
                    };
                    match db
                        .fluent()
                        .insert()
                        .into(EMAIL_VERIFICATION_COLLECTION)
                        .document_id(&verification.id)
                        .object(&verification)
                        .execute::<EmailVerification>()
                        .await
                    {
                        Ok(_) => {
                            if !mh.sign_up_verification(&s, &u, &verification.token).await {
                                error!("Couldn't send verification email");
                                return Err(database_error(&undefined()));
                            }
                        }
                        Err(e) => {
                            error!("Error saving verification: {}", e);
                            return Err(database_error(&undefined()));
                        }
                    };

                    return Ok(inserted);
                }
                .boxed()
            })
            .await
        {
            Ok(i) => return Ok(i),
            Err(e) => {
                error!("Error signing up user: {}", e);
                return Err(Error {
                    code: ErrorCode::UndefinedError,
                });
            }
        }
    }

    async fn user_by_id(&self, id: &str) -> Result<Option<User>, Error> {
        match self
            .db
            .fluent()
            .select()
            .by_id_in(USER_COLLECTION)
            .obj()
            .one(id)
            .await
        {
            Ok(f) => return Ok(f),
            Err(err) => {
                println!("Error finding user: {}", err);
                return Err(Error {
                    code: ErrorCode::UndefinedError,
                });
            }
        };
    }

    async fn verify_email(&self, token: &str) -> Result<User, Error> {
        let tkn = token.to_string();
        let res: BoxStream<FirestoreResult<EmailVerification>> = match self
            .db
            .fluent()
            .select()
            .from(EMAIL_VERIFICATION_COLLECTION)
            .filter(|q| q.field(path!(EmailVerification::token)).eq(tkn.clone()))
            .obj()
            .stream_query_with_errors()
            .await
        {
            Ok(v) => v,
            Err(e) => {
                error!("Failed to query for email verification: {}", e);
                return Err(undefined());
            }
        };

        let verifications: Vec<EmailVerification> = match res.try_collect().await {
            Ok(v) => v,
            Err(e) => {
                error!("Failed to query for email verification: {}", e);
                return Err(undefined());
            }
        };

        if verifications.is_empty() {
            error!("No verification matches token {}", tkn);
            return Err(not_found());
        }

        let user_opt: Option<User> = match self
            .db
            .fluent()
            .select()
            .by_id_in(USER_COLLECTION)
            .obj()
            .one(&verifications[0].id)
            .await
        {
            Ok(u) => u,
            Err(e) => {
                error!("Error retrieving user {}. Error {}", verifications[0].id, e);
                return Err(undefined());
            }
        };

        if user_opt.is_none() {
            error!("User {} not found", verifications[0].id);
            return Err(undefined());
        }
        let mut user = user_opt.unwrap();
        user.email_verified_at = Some(Local::now());

        let updated = self
            .db
            .fluent()
            .update()
            .fields(paths!(User::email_verified_at))
            .in_col(USER_COLLECTION)
            .document_id(&verifications[0].id)
            .object(&user)
            .execute::<User>()
            .await;

        if updated.is_err() {
            error!(
                "Failed to update user {}. Error: {}",
                user.id,
                updated.err().unwrap()
            );
            return Err(undefined());
        }

        return Ok(updated.unwrap());
    }

    async fn email_verification_by_id(&self, id: &str) -> Result<Option<EmailVerification>, Error> {
        match self
            .db
            .fluent()
            .select()
            .by_id_in(EMAIL_VERIFICATION_COLLECTION)
            .obj()
            .one(id)
            .await
        {
            Ok(f) => return Ok(f),
            Err(err) => {
                println!("Error finding email_verification: {}", err);
                return Err(Error {
                    code: ErrorCode::UndefinedError,
                });
            }
        };
    }
}
