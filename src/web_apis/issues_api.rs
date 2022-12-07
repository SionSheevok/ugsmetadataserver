use crate::sql::sql_connector;
use crate::web_apis::sqlx_result_to_our_result;
use crate::{models, UGSDatabase};
use log::info;
use rocket::http::Status;
use rocket::response::status;
use rocket::serde::json::{json, Json, Value};
use rocket_db_pools::Connection;

type Result<T> = std::result::Result<T, status::Custom<String>>;

// From MetadataServer.Controllers.IssuesController

#[rocket::get("/issues?<includeresolved>&<maxresults>", rank = 2)]
pub async fn get(
    mut db: Connection<UGSDatabase>,
    includeresolved: Option<bool>,
    maxresults: Option<i32>,
) -> Result<Json<Vec<models::IssueData>>> {
    let issues_vec_result =
        sql_connector::get_issues_filtered(&mut db, includeresolved.unwrap_or(false), maxresults)
            .await;
    sqlx_result_to_our_result(issues_vec_result).map(|t| Json(t))
}

#[rocket::get("/issues?<user>", rank = 1)]
pub async fn get_by_user(
    mut db: Connection<UGSDatabase>,
    user: String,
) -> Result<Json<Vec<models::IssueData>>> {
    let issues_vec_result = sql_connector::get_issues_by_user_name(&mut db, &user).await;
    sqlx_result_to_our_result(issues_vec_result).map(|t| Json(t))
}

#[rocket::get("/issues/<id>")]
pub async fn get_by_id(
    mut db: Connection<UGSDatabase>,
    id: i64,
) -> Result<Json<models::IssueData>> {
    let issue_result = sql_connector::get_issue(&mut db, id).await;
    match sqlx_result_to_our_result(issue_result)? {
        Some(issue) => Ok(Json(issue)),
        None => Err(status::Custom::<String>(Status::NotFound, String::new())),
    }
}

#[rocket::put("/issues/<id>", format = "application/json", data = "<issue>")]
pub async fn put(
    mut db: Connection<UGSDatabase>,
    id: i64,
    issue: Json<models::IssueUpdateData>,
) -> Result<()> {
    let issue_unwrapped = issue.into_inner();
    let result = sql_connector::update_issue(&mut db, id, &issue_unwrapped).await;
    if result.is_ok() {
        info!(r#"Issue {} successfully updated. {:?}"#, id, issue_unwrapped);
    }
    sqlx_result_to_our_result(result)
}

#[rocket::post("/issues", format = "application/json", data = "<issue>")]
pub async fn post(
    mut db: Connection<UGSDatabase>,
    issue: Json<models::IssueData>,
) -> Result<Value> {
    let issue_unwrapped = issue.into_inner();
    let issue_id_result = sql_connector::add_issue(&mut db, &issue_unwrapped).await;
    if issue_id_result.is_ok() {
        info!(
            r#"Issue {} successfully created. {:?}"#,
            issue_id_result.as_ref().unwrap(),
            issue_unwrapped
        );
    }
    sqlx_result_to_our_result(issue_id_result).map(|t| json!({ "Id": t }))
}

#[rocket::delete("/issues/<id>")]
pub async fn delete(mut db: Connection<UGSDatabase>, id: i64) -> Result<()> {
    let result = sql_connector::delete_issue(&mut db, id).await;
    if result.is_ok() {
        info!(r#"Issue {} successfully deleted."#, id);
    }
    sqlx_result_to_our_result(result)
}

// From MetadataServer.Controllers.IssueBuildsSubController
pub mod builds_sub_api {
    use crate::sql::sql_connector;
    use crate::web_apis::sqlx_result_to_our_result;
    use crate::{models, UGSDatabase};
    use log::info;
    use rocket::response::status;
    use rocket::serde::json::{json, Json, Value};
    use rocket_db_pools::Connection;

    type Result<T> = std::result::Result<T, status::Custom<String>>;

    #[rocket::get("/issues/<issue_id>/builds")]
    pub async fn get(
        mut db: Connection<UGSDatabase>,
        issue_id: i64,
    ) -> Result<Json<Vec<models::IssueBuildData>>> {
        let issue_build_data_result = sql_connector::get_builds_by_issue(&mut db, issue_id).await;
        sqlx_result_to_our_result(issue_build_data_result).map(|t| Json(t))
    }

    #[rocket::post(
        "/issues/<issue_id>/builds",
        format = "application/json",
        data = "<data>"
    )]
    pub async fn post(
        mut db: Connection<UGSDatabase>,
        issue_id: i64,
        data: Json<models::IssueBuildData>,
    ) -> Result<Value> {
        let build_id_result = sql_connector::add_build(&mut db, issue_id, &data.into_inner()).await;
        sqlx_result_to_our_result(build_id_result).map(|t| json!({ "Id": t }))
    }

    pub fn routes() -> Vec<rocket::Route> {
        rocket::routes![get, post]
    }
}

// From MetadataServer.Controllers.IssueDiagnosticsSubController
pub mod diagnostics_sub_api {
    use crate::sql::sql_connector;
    use crate::web_apis::sqlx_result_to_our_result;
    use crate::{models, UGSDatabase};
    use log::info;
    use rocket::response::status;
    use rocket::serde::json::Json;
    use rocket_db_pools::Connection;

    type Result<T> = std::result::Result<T, status::Custom<String>>;

    #[rocket::get("/issues/<issue_id>/diagnostics")]
    pub async fn get(
        mut db: Connection<UGSDatabase>,
        issue_id: i64,
    ) -> Result<Json<Vec<models::IssueDiagnosticData>>> {
        let vec_result = sql_connector::get_diagnostics(&mut db, issue_id).await;
        Ok(Json(sqlx_result_to_our_result(vec_result)?))
    }

    #[rocket::post(
        "/issues/<issue_id>/diagnostics",
        format = "application/json",
        data = "<data>"
    )]
    pub async fn post(
        mut db: Connection<UGSDatabase>,
        issue_id: i64,
        data: Json<models::IssueDiagnosticData>,
    ) -> Result<()> {
        let result = sql_connector::add_diagnostic(&mut db, issue_id, &data.into_inner()).await;
        sqlx_result_to_our_result(result)
    }

    pub fn routes() -> Vec<rocket::Route> {
        rocket::routes![get, post]
    }
}

// From MetadataServer.Controllers.IssueWatchersController
pub mod watchers_sub_api {
    use crate::sql::sql_connector;
    use crate::web_apis::sqlx_result_to_our_result;
    use crate::{models, UGSDatabase};
    use log::info;
    use rocket::response::status;
    use rocket::serde::json::Json;
    use rocket_db_pools::Connection;

    type Result<T> = std::result::Result<T, status::Custom<String>>;

    #[rocket::get("/issues/<issue_id>/watchers")]
    pub async fn get(mut db: Connection<UGSDatabase>, issue_id: i64) -> Result<Json<Vec<String>>> {
        let vec_result = sql_connector::get_watchers(&mut db, issue_id).await;
        Ok(Json(sqlx_result_to_our_result(vec_result)?))
    }

    #[rocket::post(
        "/issues/<issue_id>/watchers",
        format = "application/json",
        data = "<data>"
    )]
    pub async fn post(
        mut db: Connection<UGSDatabase>,
        issue_id: i64,
        data: Json<models::IssueWatcherData>,
    ) -> Result<()> {
        let result =
            sql_connector::add_watcher(&mut db, issue_id, &data.into_inner().user_name).await;
        sqlx_result_to_our_result(result)
    }

    #[rocket::delete(
        "/issues/<issue_id>/watchers",
        format = "application/json",
        data = "<data>"
    )]
    pub async fn delete(
        mut db: Connection<UGSDatabase>,
        issue_id: i64,
        data: Json<models::IssueWatcherData>,
    ) -> Result<()> {
        let result =
            sql_connector::remove_watcher(&mut db, issue_id, &data.into_inner().user_name).await;
        sqlx_result_to_our_result(result)
    }

    pub fn routes() -> Vec<rocket::Route> {
        rocket::routes![get, post, delete]
    }
}

pub fn routes() -> Vec<rocket::Route> {
    let base_routes = rocket::routes![get, get_by_user, get_by_id, put, post, delete];
    [
        base_routes,
        builds_sub_api::routes(),
        diagnostics_sub_api::routes(),
        watchers_sub_api::routes(),
    ]
    .concat()
}
