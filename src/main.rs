mod models;
mod sql;
mod web_apis;

use rocket_db_pools::sqlx;
use rocket_db_pools::Database;

#[derive(Database)]
#[database("ugsdb")]
pub struct UGSDatabase(sqlx::MySqlPool);

#[rocket::launch]
fn ugs_metadata_server() -> _ {
    rocket::build()
        .attach(UGSDatabase::init())
        .mount("/api", web_apis::build_api::routes())
        .mount("/api", web_apis::comment_api::routes())
        .mount("/api", web_apis::error_api::routes())
        .mount("/api", web_apis::event_api::routes())
        .mount("/api", web_apis::issuebuilds_api::routes())
        .mount("/api", web_apis::issues_api::routes())
        .mount("/api", web_apis::latest_api::routes())
        .mount("/api", web_apis::telemetry_api::routes())
        .mount("/api", web_apis::user_api::routes())
}
