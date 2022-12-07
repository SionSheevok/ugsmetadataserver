use crate::sql::sql_connector;
use crate::web_apis::sqlx_result_to_our_result;
use crate::{models, UGSDatabase};
use log::info;
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::{get, post, routes, Route};
use rocket_db_pools::Connection;

type Result<T> = std::result::Result<T, status::Custom<String>>;

// From MetadataServer.Controllers.EventController

#[get("/event?<project>&<lasteventid>")]
pub async fn get(
    mut db: Connection<UGSDatabase>,
    project: String,
    lasteventid: i64,
) -> Result<Json<Vec<models::EventData>>> {
    let events_vec_result = sql_connector::get_user_votes(&mut db, &project, lasteventid).await;
    sqlx_result_to_our_result(events_vec_result).map(|t| Json(t))
}

#[post("/event", format = "application/json", data = "<data>")]
pub async fn post(mut db: Connection<UGSDatabase>, data: Json<models::EventData>) -> Result<()> {
    let data_unwrapped = data.into_inner();
    let result = sql_connector::post_event(&mut db, &data_unwrapped).await;
    if result.is_ok() {
        info!(
            r#"User "{}" sent event "{}" for {}@{}."#,
            data_unwrapped.user_name,
            data_unwrapped.event_type,
            data_unwrapped.project,
            data_unwrapped.change
        );
    }
    sqlx_result_to_our_result(result)
}

pub fn routes() -> Vec<Route> {
    routes![get, post]
}
