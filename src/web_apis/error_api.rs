use crate::sql::sql_connector;
use crate::web_apis::sqlx_result_to_our_result;
use crate::{models, UGSDatabase};
use log::info;
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::{get, post, routes, Route};
use rocket_db_pools::Connection;

type Result<T> = std::result::Result<T, status::Custom<String>>;

// From MetadataServer.Controllers.ErrorController

#[get("/error?<records>")]
pub async fn get(
    mut db: Connection<UGSDatabase>,
    records: Option<i32>,
) -> Result<Json<Vec<models::TelemetryErrorData>>> {
    let errors_vec_results = sql_connector::get_error_data(&mut db, records.unwrap_or(10)).await;
    sqlx_result_to_our_result(errors_vec_results).map(|t| Json(t))
}

#[post(
    "/error?<version>&<ipaddress>",
    format = "application/json",
    data = "<data>"
)]
pub async fn post(
    mut db: Connection<UGSDatabase>,
    data: Json<models::TelemetryErrorData>,
    version: String,
    ipaddress: String,
) -> Result<()> {
    let data_unwrapped = data.into_inner();
    let result =
        sql_connector::post_error_data(&mut db, &data_unwrapped, &version, &ipaddress).await;
    if result.is_ok() {
        info!(r#"Error telemetry data submitted. {:?}"#, data_unwrapped);
    }
    sqlx_result_to_our_result(result)
}

pub fn routes() -> Vec<Route> {
    routes![get, post]
}
