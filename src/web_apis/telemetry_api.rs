use crate::sql::sql_connector;
use crate::web_apis::sqlx_result_to_our_result;
use crate::{models, UGSDatabase};
use log::info;
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::{post, routes, Route};
use rocket_db_pools::Connection;

type Result<T> = std::result::Result<T, status::Custom<String>>;

// From MetadataServer.Controllers.TelemetryController

#[post(
    "/telemetry?<version>&<ipaddress>",
    format = "application/json",
    data = "<data>"
)]
pub async fn post(
    mut db: Connection<UGSDatabase>,
    data: Json<models::TelemetryTimingData>,
    version: String,
    ipaddress: String,
) -> Result<()> {
    let data_unwrapped = data.into_inner();
    let result =
        sql_connector::post_telemetry_data(&mut db, &data_unwrapped, &version, &ipaddress).await;
    if result.is_ok() {
        info!(r#"Timing telemetry data submitted. {:?}"#, data_unwrapped);
    }
    sqlx_result_to_our_result(result)
}

pub fn routes() -> Vec<Route> {
    routes![post]
}
