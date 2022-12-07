use crate::sql::sql_connector;
use crate::web_apis::sqlx_result_to_our_result;
use crate::{models, UGSDatabase};
use log::info;
use rocket::http::Status;
use rocket::response::status;
use rocket::serde::json::Json;
use rocket_db_pools::Connection;

type Result<T> = std::result::Result<T, status::Custom<String>>;

// From MetadataServer.Controllers.IssueBuildsController

#[rocket::get("/issuebuilds/<buildid>")]
pub async fn get(
    mut db: Connection<UGSDatabase>,
    buildid: i64,
) -> Result<Json<models::IssueBuildData>> {
    let issue_build_data_result = sql_connector::get_build(&mut db, buildid).await;
    match sqlx_result_to_our_result(issue_build_data_result)? {
        Some(issue_build_data) => Ok(Json(issue_build_data)),
        None => Err(status::Custom(
            Status::NotFound,
            String::from(format!("No issue build with id {buildid}.")),
        )),
    }
}

#[rocket::put("/issuebuilds/<buildid>", format = "application/json", data = "<data>")]
pub async fn put(
    mut db: Connection<UGSDatabase>,
    buildid: i64,
    data: Json<models::IssueBuildUpdateData>,
) -> Result<()> {
    let data_unwrapped = data.into_inner();
    let result = sql_connector::update_build(&mut db, buildid, data_unwrapped.outcome).await;
    sqlx_result_to_our_result(result)
}

pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![get, put]
}
