use crate::sql::sql_connector;
use crate::web_apis::sqlx_result_to_our_result;
use crate::UGSDatabase;
use log::info;
use rocket::response::status;
use rocket::serde::json::{json, Value};
use rocket_db_pools::Connection;

type Result<T> = std::result::Result<T, status::Custom<String>>;

// From MetadataServer.Controllers.UserController

#[rocket::get("/user?<name>")]
pub async fn get(mut db: Connection<UGSDatabase>, name: String) -> Result<Value> {
    let user_id_result = sql_connector::find_or_add_user_id(&mut db, &name).await;
    sqlx_result_to_our_result(user_id_result).map(|t| json!({ "Id": t }))
}

pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![get]
}
