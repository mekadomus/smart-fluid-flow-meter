use async_trait::async_trait;
use tracing::error;

use crate::{
    api::{
        common::{SortDirection, DEFAULT_PAGE_SIZE},
        fluid_meter::{FluidMeter, FluidMetersInput, FluidMetersSort},
    },
    storage::{
        error::{Error, ErrorCode},
        postgres::PostgresStorage,
        FluidMeterStorage,
    },
};

#[async_trait]
impl FluidMeterStorage for PostgresStorage {
    async fn get_fluid_meters(
        &self,
        user: &String,
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
                        SELECT
                            id,
                            owner_id,
                            name,
                            status,
                            recorded_at
                        FROM fluid_meter
                        {}
                        {}
                        LIMIT $3
                    "#,
                    where_stmt, order_stmt,
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
                        SELECT
                            id,
                            owner_id,
                            name,
                            status,
                            recorded_at
                        FROM fluid_meter
                        WHERE owner_id = $1
                        {}
                        LIMIT $2
                    "#,
                    order_stmt,
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
            "INSERT INTO fluid_meter(id, owner_id, name, status, recorded_at) VALUES($1, $2, $3, $4, $5)",
        )
        .bind(&fluid_meter.id)
        .bind(&fluid_meter.owner_id)
        .bind(&fluid_meter.name)
        .bind(&fluid_meter.status)
        .bind(fluid_meter.recorded_at)
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
}
