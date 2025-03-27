use async_trait::async_trait;
use tracing::{debug, error};

use crate::{
    api::{
        common::{
            PaginatedRequest, PaginatedResponse, Pagination, SortDirection, DEFAULT_PAGE_SIZE,
        },
        fluid_meter::{
            FluidMeter,
            FluidMeterStatus::{Active, Deleted, Inactive},
            FluidMetersInput, FluidMetersSort,
        },
    },
    storage::{
        error::{undefined, Error, ErrorCode},
        postgres::PostgresStorage,
        FluidMeterStorage,
    },
};

#[async_trait]
impl FluidMeterStorage for PostgresStorage {
    async fn get_active_fluid_meters(
        &self,
        options: &PaginatedRequest,
    ) -> Result<PaginatedResponse<FluidMeter>, Error> {
        let mut meters = match &options.page_cursor {
            Some(pc) => {
                let query = r#"
                        SELECT *
                        FROM fluid_meter
                        WHERE status = $1 AND id > $2
                        ORDER BY id
                        LIMIT $3
                    "#;

                match sqlx::query_as(&query)
                    .bind(Active)
                    .bind(&pc)
                    .bind((options.page_size + 1) as i32)
                    .fetch_all(&self.pool)
                    .await
                {
                    Ok(found) => found,
                    Err(err) => {
                        error!("Error getting fluid_meters: {}", err);
                        return Err(Error {
                            code: ErrorCode::UndefinedError,
                        });
                    }
                }
            }
            None => {
                let query = r#"
                        SELECT *
                        FROM fluid_meter
                        WHERE status = $1
                        ORDER BY id
                        LIMIT $2
                    "#;

                match sqlx::query_as(&query)
                    .bind(Active)
                    .bind((options.page_size + 1) as i32)
                    .fetch_all(&self.pool)
                    .await
                {
                    Ok(found) => found,
                    Err(err) => {
                        error!("Error getting fluid_meters: {}", err);
                        return Err(Error {
                            code: ErrorCode::UndefinedError,
                        });
                    }
                }
            }
        };

        let mut has_more = false;
        if (meters.len() as u8) > options.page_size {
            has_more = true;
            meters.pop();
        }
        return Ok(PaginatedResponse {
            items: meters,
            pagination: Pagination {
                has_more,
                has_less: options.page_cursor.is_some(),
            },
        });
    }

    async fn get_fluid_meters(
        &self,
        user: &str,
        options: &FluidMetersInput,
    ) -> Result<Vec<FluidMeter>, Error> {
        let page_size = options.page_size.unwrap_or(*DEFAULT_PAGE_SIZE) as i32;
        let sort_field = match &options.sort {
            Some(s) => match s {
                FluidMetersSort::Id => "id",
                FluidMetersSort::Name => "name",
            },
            None => "id",
        };
        let sort_direction = match &options.sort_direction {
            Some(s) => match s {
                SortDirection::Asc => "ASC",
                SortDirection::Desc => "DESC",
            },
            None => "ASC",
        };

        let status_filter = match &options.status {
            Some(s) => format!("AND status = '{}'", s),
            None => format!("AND status != '{}'", Deleted),
        };

        match options.page_cursor.clone() {
            Some(cursor) => {
                let where_stmt = if sort_direction == "ASC" {
                    format!("WHERE owner_id = $1 AND {} > $2", sort_field)
                } else {
                    format!("WHERE owner_id = $1 AND {} < $2", sort_field)
                };
                let order_stmt = format!("ORDER BY {} {}", sort_field, sort_direction);
                let query = format!(
                    r#"
                        SELECT *
                        FROM fluid_meter
                        {}
                        {}
                        {}
                        LIMIT $3
                    "#,
                    where_stmt, status_filter, order_stmt,
                );

                match sqlx::query_as(&query)
                    .bind(user)
                    .bind(cursor)
                    .bind(page_size)
                    .fetch_all(&self.pool)
                    .await
                {
                    Ok(found) => {
                        return Ok(found);
                    }
                    Err(err) => {
                        error!("Error getting fluid_meters: {}", err);
                        return Err(Error {
                            code: ErrorCode::UndefinedError,
                        });
                    }
                };
            }
            None => {
                let order_stmt = format!("ORDER BY {} {}", sort_field, sort_direction);
                let query = format!(
                    r#"
                        SELECT *
                        FROM fluid_meter
                        WHERE owner_id = $1
                        {}
                        {}
                        LIMIT $2
                    "#,
                    status_filter, order_stmt,
                );

                match sqlx::query_as(&query)
                    .bind(user)
                    .bind(page_size)
                    .fetch_all(&self.pool)
                    .await
                {
                    Ok(found) => {
                        return Ok(found);
                    }
                    Err(err) => {
                        error!("Error getting fluid_meters: {}", err);
                        return Err(Error {
                            code: ErrorCode::UndefinedError,
                        });
                    }
                };
            }
        };
    }

    async fn insert_fluid_meter(&self, fluid_meter: &FluidMeter) -> Result<FluidMeter, Error> {
        match sqlx::query(
            "INSERT INTO fluid_meter(id, owner_id, name, status, recorded_at, updated_at) VALUES($1, $2, $3, $4, $5, $6)",
        )
        .bind(&fluid_meter.id)
        .bind(&fluid_meter.owner_id)
        .bind(&fluid_meter.name)
        .bind(&fluid_meter.status)
        .bind(fluid_meter.recorded_at)
        .bind(fluid_meter.updated_at)
        .execute(&self.pool)
        .await
        {
            Ok(_) => return Ok(fluid_meter.clone()),
            Err(err) => {
                error!("Error: {}", err);
                return Err(Error {
                    code: ErrorCode::UndefinedError,
                });
            }
        };
    }

    async fn delete_fluid_meter(&self, id: &str) -> Result<(), Error> {
        debug!("delete_fluid_meter: {}", &id);
        match sqlx::query("UPDATE fluid_meter SET status = $1 WHERE id = $2")
            .bind(&Deleted)
            .bind(&id)
            .execute(&self.pool)
            .await
        {
            Ok(_) => return Ok(()),
            Err(e) => {
                error!("Error deleting fluid meter: {}", e);
                return undefined();
            }
        };
    }

    async fn get_fluid_meter_by_id(&self, id: &str) -> Result<Option<FluidMeter>, Error> {
        debug!("get_fluid_meter_by_id: {}", &id);
        match sqlx::query_as("SELECT * FROM fluid_meter WHERE id = $1")
            .bind(&id)
            .fetch_one(&self.pool)
            .await
        {
            Ok(f) => return Ok(Some(f)),
            Err(e) => {
                match e {
                    sqlx::Error::RowNotFound => {
                        return Ok(None);
                    }
                    _ => {}
                }

                error!("Error getting fluid_meter by id. {}", e);
                return undefined();
            }
        };
    }

    async fn is_fluid_meter_owner(
        &self,
        fluid_meter_id: &str,
        account_id: &str,
    ) -> Result<bool, Error> {
        match sqlx::query("SELECT * FROM fluid_meter WHERE id = $1 AND owner_id = $2")
            .bind(&fluid_meter_id)
            .bind(&account_id)
            .fetch_one(&self.pool)
            .await
        {
            Ok(_) => return Ok(true),
            Err(e) => {
                match e {
                    sqlx::Error::RowNotFound => {
                        return Ok(false);
                    }
                    _ => {}
                }

                error!("Error getting fluid_meter for user by id. {}", e);
                return undefined();
            }
        };
    }

    async fn activate_fluid_meter(&self, meter_id: &str) -> Result<(), Error> {
        match sqlx::query("UPDATE fluid_meter SET status = $1 WHERE id = $2")
            .bind(&Active)
            .bind(&meter_id)
            .execute(&self.pool)
            .await
        {
            Ok(_) => return Ok(()),
            Err(e) => {
                error!(
                    "Error activating fluid_meter id. {}. Error: {}",
                    meter_id, e
                );
                return undefined();
            }
        };
    }

    async fn deactivate_fluid_meter(&self, meter_id: &str) -> Result<(), Error> {
        match sqlx::query("UPDATE fluid_meter SET status = $1 WHERE id = $2")
            .bind(&Inactive)
            .bind(&meter_id)
            .execute(&self.pool)
            .await
        {
            Ok(_) => return Ok(()),
            Err(e) => {
                error!(
                    "Error deactivating fluid_meter id. {}. Error: {}",
                    meter_id, e
                );
                return undefined();
            }
        };
    }
}
