use async_trait::async_trait;
use firestore::{path, FirestoreQueryDirection};
use tracing::error;

use crate::{
    api::{
        common::{SortDirection, DEFAULT_PAGE_SIZE},
        fluid_meter::{FluidMeter, FluidMetersInput, FluidMetersSort},
    },
    storage::{
        error::{Error, ErrorCode},
        firestore::FirestoreStorage,
        FluidMeterStorage,
    },
};

const FLUID_METER_COLLECTION: &'static str = "fluid_meter";

#[async_trait]
impl FluidMeterStorage for FirestoreStorage {
    async fn get_fluid_meters(
        &self,
        user: &String,
        options: &FluidMetersInput,
    ) -> Result<Vec<FluidMeter>, Error> {
        let page_size = options.page_size.unwrap_or(*DEFAULT_PAGE_SIZE);
        let sort_field = match &options.sort {
            Some(s) => match s {
                FluidMetersSort::Id => path!(FluidMeter::id),
                FluidMetersSort::Name => path!(FluidMeter::name),
            },
            None => path!(FluidMeter::id),
        };
        let sort_direction = match &options.sort_direction {
            Some(s) => match s {
                SortDirection::Asc => FirestoreQueryDirection::Ascending,
                SortDirection::Desc => FirestoreQueryDirection::Descending,
            },
            None => FirestoreQueryDirection::Ascending,
        };

        match self
            .db
            .fluent()
            .select()
            .from(FLUID_METER_COLLECTION)
            .filter(|q| {
                let mut filters = vec![q.field(path!(FluidMeter::owner_id)).eq(user)];
                if options.page_cursor.is_some() {
                    let cursor = options.page_cursor.clone().unwrap();
                    if sort_direction == FirestoreQueryDirection::Ascending {
                        filters.push(q.field(sort_field.clone()).greater_than(cursor));
                    } else {
                        filters.push(q.field(sort_field.clone()).less_than(cursor));
                    }
                }
                q.for_all(filters)
            })
            .order_by([(sort_field, sort_direction)])
            .limit(page_size.into())
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

    async fn insert_fluid_meter(&self, fluid_meter: &FluidMeter) -> Result<FluidMeter, Error> {
        let inserted: FluidMeter = match self
            .db
            .fluent()
            .insert()
            .into(FLUID_METER_COLLECTION)
            .document_id(&fluid_meter.id)
            .object(fluid_meter)
            .execute()
            .await
        {
            Ok(inserted) => inserted,
            Err(err) => {
                error!("Error: {}", err);
                return Err(Error {
                    code: ErrorCode::UndefinedError,
                });
            }
        };

        Ok(inserted)
    }
}
