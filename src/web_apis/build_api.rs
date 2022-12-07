use crate::sql::sql_connector;
use crate::web_apis::sqlx_result_to_our_result;
use crate::{models, UGSDatabase};
use log::info;
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::{get, post, routes, Route};
use rocket_db_pools::Connection;

type Result<T> = std::result::Result<T, status::Custom<String>>;

// From MetadataServer.Controllers.BuildController

#[get("/build?<project>&<lastbuildid>")]
pub async fn get(
    mut db: Connection<UGSDatabase>,
    project: String,
    lastbuildid: i64,
) -> Result<Json<Vec<models::BuildData>>> {
    let builds_vec_result = sql_connector::get_builds(&mut db, &project, lastbuildid).await;
    sqlx_result_to_our_result(builds_vec_result).map(|t| Json(t))
}

#[post("/build", format = "application/json", data = "<build>")]
pub async fn post(mut db: Connection<UGSDatabase>, build: Json<models::BuildData>) -> Result<()> {
    let build_unwrapped = build.into_inner();
    let result = sql_connector::post_build(&mut db, &build_unwrapped).await;
    if result.is_ok() {
        info!(
            r#"Build badge "{}" successfully updated for {}@{} to status "{}"."#,
            build_unwrapped.build_type,
            build_unwrapped.project,
            build_unwrapped.change_number,
            build_unwrapped.result
        );
    }
    sqlx_result_to_our_result(result)
}

pub fn routes() -> Vec<Route> {
    routes![get, post]
}
