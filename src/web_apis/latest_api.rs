use crate::sql::sql_connector;
use crate::web_apis::sqlx_result_to_our_result;
use crate::{models, UGSDatabase};
use log::info;
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::{get, routes, Route};
use rocket_db_pools::Connection;

type Result<T> = std::result::Result<T, status::Custom<String>>;

// From MetadataServer.Controllers.LatestController

#[get("/latest?<project>")]
pub async fn get(
    mut db: Connection<UGSDatabase>,
    project: Option<String>,
) -> Result<Json<models::LatestData>> {
    let latest_data_result = sql_connector::get_last_ids(&mut db, project.as_deref()).await;
    sqlx_result_to_our_result(latest_data_result).map(|t| Json(t))
}

pub fn routes() -> Vec<Route> {
    routes![get]
}
