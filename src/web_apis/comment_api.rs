use crate::sql::sql_connector;
use crate::web_apis::sqlx_result_to_our_result;
use crate::{models, UGSDatabase};
use log::info;
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::{get, post, routes, Route};
use rocket_db_pools::Connection;

type Result<T> = std::result::Result<T, status::Custom<String>>;

// From MetadataServer.Controllers.CommentController

#[get("/comment?<project>&<lastcommentid>")]
pub async fn get(
    mut db: Connection<UGSDatabase>,
    project: String,
    lastcommentid: i64,
) -> Result<Json<Vec<models::CommentData>>> {
    info!(
        r#"Received call to get comments newer than id {} for project {}."#,
        lastcommentid, &project
    );
    let comments_vec_result = sql_connector::get_comments(&mut db, &project, lastcommentid).await;
    sqlx_result_to_our_result(comments_vec_result).map(|t| Json(t))
}

#[post("/comment", format = "application/json", data = "<comment>")]
pub async fn post(
    mut db: Connection<UGSDatabase>,
    comment: Json<models::CommentData>,
) -> Result<()> {
    let comment_unwrapped = comment.into_inner();
    let result = sql_connector::post_comment(&mut db, &comment_unwrapped).await;
    if result.is_ok() {
        info!(
            r#"Comment by user "{}" successfully updated for {}@{} to: "{}"."#,
            comment_unwrapped.user_name,
            comment_unwrapped.project,
            comment_unwrapped.change_number,
            comment_unwrapped.text
        );
    }
    sqlx_result_to_our_result(result)
}

pub fn routes() -> Vec<Route> {
    routes![get, post]
}
