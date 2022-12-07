pub mod build_api;
pub mod comment_api;
pub mod error_api;
pub mod event_api;
pub mod issuebuilds_api;
pub mod issues_api;
pub mod latest_api;
pub mod telemetry_api;
pub mod user_api;

use rocket::http::Status;
use rocket::response::status;

pub fn sqlx_error_to_custom_status(sqlx_error: &sqlx::Error) -> status::Custom<String> {
    log::warn!("Database error: {}", sqlx_error.to_string());
    status::Custom(
        Status::InternalServerError,
        String::from("Database error occurred."),
    )
}

pub fn sqlx_result_to_our_result<T>(
    sqlx_result: Result<T, sqlx::Error>,
) -> Result<T, status::Custom<String>> {
    sqlx_result.map_err(|e| sqlx_error_to_custom_status(&e))
}
