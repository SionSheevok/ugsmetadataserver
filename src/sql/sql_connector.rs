use crate::{models, UGSDatabase};
use rocket_db_pools::sqlx;
use rocket_db_pools::sqlx::Acquire;
use rocket_db_pools::sqlx::FromRow;
use rocket_db_pools::Connection;

type Result<T> = std::result::Result<T, sqlx::Error>;

const ISSUE_SUMMARY_MAX_LENGTH: usize = 200;

// Public Functions:

pub async fn get_last_ids(
    sql_connection: &mut Connection<UGSDatabase>,
    project: Option<&str>,
) -> Result<models::LatestData> {
    let project_like_string = get_project_like_string(project);

    let last_event_id = sqlx::query_scalar::<_, i64>(
        r#"WITH user_votes AS (SELECT UserVotes.Id, UserVotes.Changelist FROM ugs_db.UserVotes 
        INNER JOIN ugs_db.Projects ON Projects.Id = UserVotes.ProjectId 
        WHERE Projects.Name LIKE ? GROUP BY Changelist ORDER BY Changelist DESC LIMIT 100) 
        SELECT Id FROM user_votes ORDER BY user_votes.Changelist ASC LIMIT 1"#,
    )
    .bind(&project_like_string)
    .fetch_optional(&mut *(*sql_connection))
    .await?
    .unwrap_or(0);

    let last_comment_id =
    sqlx::query_scalar::<_, i64>(r#"WITH comments AS (SELECT Comments.Id, Comments.ChangeNumber FROM ugs_db.Comments 
                    INNER JOIN ugs_db.Projects ON Projects.Id = Comments.ProjectId 
                    WHERE Projects.Name LIKE ? GROUP BY ChangeNumber ORDER BY ChangeNumber DESC LIMIT 100) 
                    SELECT Id FROM comments ORDER BY comments.ChangeNumber ASC LIMIT 1"#)
                        .bind(&project_like_string)
                        .fetch_optional(&mut *(*sql_connection))
                        .await?
                        .unwrap_or(0);

    let last_build_id =
    sqlx::query_scalar::<_, i64>(r#"WITH badges AS (SELECT Badges.Id, Badges.ChangeNumber FROM ugs_db.Badges 
                        INNER JOIN ugs_db.Projects ON Projects.Id = Badges.ProjectId 
                        WHERE Projects.Name LIKE ? GROUP BY ChangeNumber ORDER BY ChangeNumber DESC LIMIT 100) 
                        SELECT Id FROM badges ORDER BY badges.ChangeNumber ASC LIMIT 1"#)
                                .bind(&project_like_string)
                                .fetch_optional(&mut *(*sql_connection))
                                .await?
                                .unwrap_or(0);

    Ok(models::LatestData {
        last_event_id,
        last_build_id,
        last_comment_id,
    })
}

pub async fn get_user_votes(
    sql_connection: &mut Connection<UGSDatabase>,
    project: &str,
    last_event_id: i64,
) -> Result<Vec<models::EventData>> {
    let project_like_string = get_project_like_string(Some(project));
    let event_data_vec: Vec<models::EventData> = sqlx::query_as::<_, models::EventData>(
        r#"SELECT UserVotes.Id, UserVotes.Changelist AS `Change`, UserVotes.UserName, UserVotes.Verdict AS `EventType`, UserVotes.Project FROM ugs_db.UserVotes INNER JOIN ugs_db.Projects ON Projects.Id = UserVotes.ProjectId WHERE UserVotes.Id > ? AND Projects.Name LIKE ? ORDER BY UserVotes.Id"#
        )
        .bind(last_event_id)
        .bind(project_like_string)
        .fetch_all(&mut *(*sql_connection)).await?
        .into_iter()
        .filter(|event_data| event_data.project.is_empty() || event_data.project == project)
        .collect();

    Ok(event_data_vec)
}

pub async fn get_comments(
    sql_connection: &mut Connection<UGSDatabase>,
    project: &str,
    last_comment_id: i64,
) -> Result<Vec<models::CommentData>> {
    let project_like_string = get_project_like_string(Some(project));
    let comment_data_vec: Vec<models::CommentData> = sqlx::query_as::<_, models::CommentData>(
        r#"SELECT Comments.Id, Comments.ChangeNumber, Comments.UserName, Comments.Text, Comments.Project FROM ugs_db.Comments INNER JOIN ugs_db.Projects ON Projects.Id = Comments.ProjectId WHERE Comments.Id > ? AND Projects.Name LIKE ? ORDER BY Comments.Id"#
        )
        .bind(last_comment_id)
        .bind(&project_like_string)
        .fetch_all(&mut *(*sql_connection)).await?
        .into_iter()
        .filter(|comment_data| comment_data.project.is_empty() || comment_data.project == project)
        .collect();

    Ok(comment_data_vec)
}

pub async fn get_builds(
    sql_connection: &mut Connection<UGSDatabase>,
    project: &str,
    last_build_id: i64,
) -> Result<Vec<models::BuildData>> {
    let project_like_string = get_project_like_string(Some(project));
    let build_data_vec = sqlx::query_as::<_, models::BuildData>(
        r#"SELECT Badges.Id, Badges.ChangeNumber, Badges.BuildType, Badges.Result, Badges.Url, Projects.Name AS `Project`, Badges.ArchivePath FROM ugs_db.Badges INNER JOIN ugs_db.Projects ON Projects.Id = Badges.ProjectId WHERE Badges.Id > ? AND Projects.Name LIKE ? ORDER BY Badges.Id"#
        )
        .bind(last_build_id)
        .bind(project_like_string)
        .fetch_all(&mut *(*sql_connection)).await?;
    Ok(build_data_vec
        .into_iter()
        .filter(|build_data| {
            build_data.project.is_empty()
                || build_data.project == project
                || matches_wildcard(&build_data.project, project)
        })
        .collect::<Vec<models::BuildData>>())
}

pub async fn get_error_data(
    sql_connection: &mut Connection<UGSDatabase>,
    records: i32,
) -> Result<Vec<models::TelemetryErrorData>> {
    sqlx::query_as::<_, models::TelemetryErrorData>(
        r#"SELECT Id, Type, Text, UserName, Project, Timestamp, Version, IpAddress FROM ugs_db.Errors ORDER BY Id DESC LIMIT ?"#
        )
        .bind(records)
        .fetch_all(&mut *(*sql_connection))
        .await
}

pub async fn post_build(
    sql_connection: &mut Connection<UGSDatabase>,
    build: &models::BuildData,
) -> Result<()> {
    let project_id = try_insert_and_get_project(sql_connection, &build.project).await?;
    sqlx::query(&r#"INSERT INTO ugs_db.Badges (ChangeNumber, BuildType, Result, URL, ArchivePath, ProjectId) VALUES (?, ?, ?, ?, ?, ?)"#)
        .bind(build.change_number)
        .bind(&build.build_type)
        .bind(build.result.to_string())
        .bind(&build.url)
        .bind(&build.archive_path)
        .bind(project_id)
        .execute(&mut *(*sql_connection)).await?;
    Ok(())
}

pub async fn post_event(
    sql_connection: &mut Connection<UGSDatabase>,
    event: &models::EventData,
) -> Result<()> {
    let project_id = try_insert_and_get_project(sql_connection, &event.project).await?;
    sqlx::query(&r#"INSERT INTO ugs_db.UserVotes (Changelist, UserName, Verdict, Project, ProjectId) VALUES (?, ?, ?, ?, ?)"#)
        .bind(event.change)
        .bind(&event.user_name)
        .bind(event.event_type.to_string())
        .bind(&event.project)
        .bind(project_id)
        .execute(&mut *(*sql_connection)).await?;
    Ok(())
}

pub async fn post_comment(
    sql_connection: &mut Connection<UGSDatabase>,
    comment: &models::CommentData,
) -> Result<()> {
    let project_id = try_insert_and_get_project(sql_connection, &comment.project).await?;
    sqlx::query(&r#"INSERT INTO ugs_db.Comments (ChangeNumber, UserName, Text, Project, ProjectId) VALUES (?, ?, ?, ?, ?)"#)
        .bind(comment.change_number)
        .bind(&comment.user_name)
        .bind(&comment.text)
        .bind(&comment.project)
        .bind(project_id)
        .execute(&mut *(*sql_connection)).await?;
    Ok(())
}

pub async fn post_telemetry_data(
    sql_connection: &mut Connection<UGSDatabase>,
    data: &models::TelemetryTimingData,
    version: &str,
    ip_address: &str,
) -> Result<()> {
    let project_id = try_insert_and_get_project(sql_connection, &data.project).await?;
    sqlx::query(&r#"INSERT INTO ugs_db.Telemetry_v2 (Action, Result, UserName, Project, Timestamp, Duration, Version, IpAddress, ProjectId) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"#)
        .bind(&data.action)
        .bind(&data.result)
        .bind(&data.user_name)
        .bind(&data.project)
        .bind(data.timestamp)
        .bind(data.duration)
        .bind(version)
        .bind(ip_address)
        .bind(project_id)
        .execute(&mut *(*sql_connection)).await?;
    Ok(())
}

pub async fn post_error_data(
    sql_connection: &mut Connection<UGSDatabase>,
    data: &models::TelemetryErrorData,
    version: &str,
    ip_address: &str,
) -> Result<()> {
    sqlx::query(&r#"INSERT INTO ugs_db.Telemetry_v2 (Action, Result, UserName, Project, Timestamp, Duration, Version, IpAddress, ProjectId) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"#)
        .bind(&data.error_type)
        .bind(&data.text)
        .bind(&data.user_name)
        //
        .bind(&data.project)
        .bind(match &data.project { Some(project_name) => Some(try_insert_and_get_project(sql_connection, &project_name).await?), None => None })
        //
        .bind(data.timestamp)
        .bind(version)
        .bind(ip_address)
        .execute(&mut *(*sql_connection)).await?;
    Ok(())
}

pub async fn find_or_add_user_id(
    sql_connection: &mut Connection<UGSDatabase>,
    name: &str,
) -> Result<Option<i64>> {
    if name.is_empty() {
        return Ok(None);
    }

    let normalized_name = normalize_user_name(name);

    // Try to get the id if it already exists.
    {
        let id_opt = sqlx::query_scalar::<_, i64>(r#"SELECT Id FROM ugs_db.Users WHERE Name = ?"#)
            .bind(&normalized_name)
            .fetch_optional(&mut *(*sql_connection))
            .await?;

        if id_opt.is_some() {
            return Ok(id_opt);
        }
    }

    // Otherwise, start a transaction and try to create the row and get the id.
    let mut transaction = sql_connection.begin().await?;

    sqlx::query(r#"INSERT IGNORE INTO ugs_db.Users (Name) VALUES (?)"#)
        .bind(&normalized_name)
        .execute(&mut transaction)
        .await?;

    let id = sqlx::query_scalar::<_, i64>(r#"SELECT Id FROM ugs_db.Users WHERE Name = ?"#)
        .bind(&normalized_name)
        .fetch_one(&mut transaction)
        .await?;

    transaction.commit().await?;

    Ok(Some(id))
}

pub async fn add_issue(
    sql_connection: &mut Connection<UGSDatabase>,
    issue: &models::IssueData,
) -> Result<i64> {
    let id = sqlx::query_scalar::<_, i64>(r#"INSERT INTO ugs_db.Issues (Project, Summary, OwnerId, CreatedAt, FixChange) VALUES (?, ?, ?, UTC_TIMESTAMP(), 0)"#)
        .bind(&issue.project)
        .bind(sanitize_text(&issue.summary, ISSUE_SUMMARY_MAX_LENGTH))
        //
        .bind(find_or_add_user_id(sql_connection, &issue.owner).await?)
        //
        .fetch_one(&mut *(*sql_connection)).await?;

    // TODO: Figure out how to get the id properly?

    Ok(id)
}

pub async fn get_issue(
    sql_connection: &mut Connection<UGSDatabase>,
    issue_id: i64,
) -> Result<Option<models::IssueData>> {
    let issue_data_vec =
        get_issues_internal(sql_connection, Some(issue_id), None, true, None).await?;
    if issue_data_vec.is_empty() {
        Ok(None)
    } else {
        Ok(Some(issue_data_vec[0].clone()))
    }
}

pub async fn get_issues_filtered(
    sql_connection: &mut Connection<UGSDatabase>,
    include_resolved: bool,
    num_results: Option<i32>,
) -> Result<Vec<models::IssueData>> {
    get_issues_internal(sql_connection, None, None, include_resolved, num_results).await
}

pub async fn get_issues_by_user_name(
    sql_connection: &mut Connection<UGSDatabase>,
    user_name: &str,
) -> Result<Vec<models::IssueData>> {
    get_issues_internal(sql_connection, None, Some(user_name), false, None).await
}

async fn get_issues_internal(
    sql_connection: &mut Connection<UGSDatabase>,
    issue_id: Option<i64>,
    user_name: Option<&str>,
    include_resolved: bool,
    num_results: Option<i32>,
) -> Result<Vec<models::IssueData>> {
    let user_id = match user_name {
        Some(s) => find_or_add_user_id(sql_connection, s).await?,
        None => None,
    };
    let mut query_builder: sqlx::QueryBuilder<sqlx::MySql> = sqlx::QueryBuilder::new("SELECT");
    query_builder.push(" Issues.Id, Issues.CreatedAt, UTC_TIMESTAMP(), Issues.Project, Issues.Summary, OwnerUsers.Name, NominatedByUsers.Name, Issues.AcknowledgedAt, Issues.FixChange, Issues.ResolvedAt");
    if user_name.is_some() {
        query_builder.push(", IssueWatchers.UserId");
    };
    query_builder.push(" FROM ugs_db.Issues");
    query_builder.push(" LEFT JOIN ugs_db.Users AS OwnerUsers ON OwnerUsers.Id = Issues.OwnerId");
    query_builder.push(
        " LEFT JOIN ugs_db.Users AS NominatedByUsers ON NominatedByUsers.Id = Issues.NominatedById",
    );
    if user_name.is_some() {
        query_builder.push(" LEFT JOIN ugs_db.IssueWatchers ON IssueWatchers.IssueId = Issues.Id AND IssueWatchers.UserId = ").push_bind(user_id.unwrap());
    }
    if issue_id.is_some() {
        query_builder
            .push(" WHERE Issues.Id = ")
            .push_bind(issue_id.unwrap());
    } else if !include_resolved {
        query_builder.push(" WHERE Issues.ResolvedAt IS NULL");
    }
    if num_results.is_some() {
        query_builder
            .push(" ORDER BY Issues.Id DESC LIMIT ")
            .push_bind(num_results.unwrap());
    }

    // TODO: The Notify field of IssueData requires some special handling, it looks like.

    query_builder
        .build()
        .map(|row: sqlx::mysql::MySqlRow| models::IssueData::from_row(&row).unwrap())
        .fetch_all(&mut *(*sql_connection))
        .await
}

pub async fn update_issue(
    sql_connection: &mut Connection<UGSDatabase>,
    issue_id: i64,
    issue: &models::IssueUpdateData,
) -> Result<()> {
    let mut query_builder: sqlx::QueryBuilder<sqlx::MySql> =
        sqlx::QueryBuilder::new("UPDATE ugs_db.Issues SET ");
    {
        let mut separated_builder = query_builder.separated(",");
        if !issue.summary.is_empty() {
            separated_builder
                .push("Summary=")
                .push_bind(sanitize_text(&issue.summary, ISSUE_SUMMARY_MAX_LENGTH));
        }
        if !issue.owner.is_empty() {
            separated_builder
                .push("OwnerId=")
                .push_bind(find_or_add_user_id(sql_connection, &issue.owner).await?);
        }
        if !issue.nominated_by.is_empty() {
            separated_builder
                .push("NominatedById=")
                .push_bind(find_or_add_user_id(sql_connection, &issue.nominated_by).await?);
        }
        if issue.acknowledged.is_some() {
            separated_builder
                .push("AcknowledgedAt=")
                .push(if issue.acknowledged.unwrap() {
                    "UTC_TIMESTAMP()"
                } else {
                    "NULL"
                });
        }
        if issue.fix_change.is_some() {
            separated_builder
                .push("FixChange=")
                .push_bind(issue.fix_change.unwrap());
        }
        if issue.resolved.is_some() {
            separated_builder
                .push("ResolvedAt=")
                .push(if issue.resolved.unwrap() {
                    "UTC_TIMESTAMP()"
                } else {
                    "NULL"
                });
        }
    }
    query_builder.push(" WHERE id = ").push_bind(issue_id);

    query_builder
        .build()
        .execute(&mut *(*sql_connection))
        .await?;
    Ok(())
}

pub fn sanitize_text(text: &str, length: usize) -> String {
    if text.len() > length {
        let truncated_text = &text[0..length];
        let newline_idx_opt = truncated_text.rfind("\n");
        const ELLIPSES: &str = "...";
        return match newline_idx_opt {
            Some(newline_idx) => String::from([&text[0..(newline_idx + 1)], ELLIPSES].concat()),
            None => String::from([&text[0..(length - 3)], ELLIPSES].concat()),
        };
    }
    String::from(text)
}

pub async fn delete_issue(
    sql_connection: &mut Connection<UGSDatabase>,
    issue_id: i64,
) -> Result<()> {
    let mut transaction = sql_connection.begin().await?;

    sqlx::query(r#"DELETE FROM ugs_db.IssueWatchers WHERE IssueId = ?"#)
        .bind(issue_id)
        .execute(&mut transaction)
        .await?;

    sqlx::query(r#"DELETE FROM ugs_db.IssueBuilds WHERE IssueId = ?"#)
        .bind(issue_id)
        .execute(&mut transaction)
        .await?;

    sqlx::query(r#"DELETE FROM ugs_db.Issues WHERE IssueId = ?"#)
        .bind(issue_id)
        .execute(&mut transaction)
        .await?;

    transaction.commit().await?;

    Ok(())
}

pub async fn add_diagnostic(
    sql_connection: &mut Connection<UGSDatabase>,
    issue_id: i64,
    diagnostic: &models::IssueDiagnosticData,
) -> Result<()> {
    sqlx::query(r#"INSERT INTO ugs_db.IssueDiagnostics (IssueId, BuildId, Message, Url) VALUES (?, ?, ?, ?)"#)
        .bind(issue_id)
        .bind(diagnostic.build_id)
        .bind(sanitize_text(&diagnostic.message, 1000))
        .bind(&diagnostic.url)
        .execute(&mut *(*sql_connection))
        .await?;
    Ok(())
}

pub async fn get_diagnostics(
    sql_connection: &mut Connection<UGSDatabase>,
    issue_id: i64,
) -> Result<Vec<models::IssueDiagnosticData>> {
    sqlx::query_as::<_, models::IssueDiagnosticData>(
        r#"SELECT BuildId, Message, Url FROM ugs_db.IssueDiagnostics 
        WHERE IssueDiagnostics.IssueId = ?"#,
    )
    .bind(issue_id)
    .fetch_all(&mut *(*sql_connection))
    .await
}

pub async fn add_watcher(
    sql_connection: &mut Connection<UGSDatabase>,
    issue_id: i64,
    user_name: &str,
) -> Result<()> {
    sqlx::query(r#"INSERT IGNORE INTO ugs_db.IssueWatchers (IssueId, UserId) VALUES (?, ?)"#)
        .bind(issue_id)
        .bind(find_or_add_user_id(sql_connection, user_name).await?)
        .execute(&mut *(*sql_connection))
        .await?;
    Ok(())
}

pub async fn get_watchers(
    sql_connection: &mut Connection<UGSDatabase>,
    issue_id: i64,
) -> Result<Vec<String>> {
    sqlx::query_scalar(
        r#"SELECT Users.Name FROM ugs_db.IssueWatchers 
        LEFT JOIN ugs_db.Users ON IssueWatchers.UserId = Users.Id 
        WHERE IssueWatchers.IssueId = ?"#,
    )
    .bind(issue_id)
    .fetch_all(&mut *(*sql_connection))
    .await
}

pub async fn remove_watcher(
    sql_connection: &mut Connection<UGSDatabase>,
    issue_id: i64,
    user_name: &str,
) -> Result<()> {
    sqlx::query(r#"DELETE FROM ugs_db.IssueWatchers WHERE IssueId = ? AND UserId = ?"#)
        .bind(issue_id)
        .bind(find_or_add_user_id(sql_connection, user_name).await?)
        .execute(&mut *(*sql_connection))
        .await?;
    Ok(())
}

pub async fn add_build(
    sql_connection: &mut Connection<UGSDatabase>,
    issue_id: i64,
    build: &models::IssueBuildData,
) -> Result<i64> {
    sqlx::query_scalar::<_, i64>(r#"INSERT INTO ugs_db.IssueBuilds (IssueId, Stream, `Change`, JobName, JobUrl, JobStepName, JobStepUrl, ErrorUrl, Outcome) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"#)
        .bind(issue_id)
        .bind(&build.stream)
        .bind(build.change)
        .bind(&build.job_name)
        .bind(&build.job_url)
        .bind(&build.job_step_name)
        .bind(&build.job_step_url)
        .bind(&build.error_url)
        .bind(build.outcome)
        .fetch_one(&mut *(*sql_connection))
        .await
}

pub async fn get_builds_by_issue(
    sql_connection: &mut Connection<UGSDatabase>,
    issue_id: i64,
) -> Result<Vec<models::IssueBuildData>> {
    sqlx::query_as::<_, models::IssueBuildData>(r#"SELECT IssueBuilds.Id, IssueBuilds.Stream, IssueBuilds.Change, IssueBuilds.JobName, IssueBuilds.JobUrl, IssueBuilds.JobStepName, IssueBuilds.JobStepUrl, IssueBuilds.ErrorUrl, IssueBuilds.Outcome FROM ugs_db.IssueBuilds WHERE IssueBuilds.IssueId = ?"#)
        .bind(issue_id)
        .fetch_all(&mut *(*sql_connection))
        .await
}

pub async fn get_build(
    sql_connection: &mut Connection<UGSDatabase>,
    build_id: i64,
) -> Result<Option<models::IssueBuildData>> {
    sqlx::query_as::<_, models::IssueBuildData>(r#"SELECT IssueBuilds.Id, IssueBuilds.Stream, IssueBuilds.Change, IssueBuilds.JobName, IssueBuilds.JobUrl, IssueBuilds.JobStepName, IssueBuilds.JobStepUrl, IssueBuilds.ErrorUrl, IssueBuilds.Outcome FROM ugs_db.IssueBuilds WHERE IssueBuilds.Id = ?"#)
        .bind(build_id)
        .fetch_optional(&mut *(*sql_connection))
        .await
}

pub async fn update_build(
    sql_connection: &mut Connection<UGSDatabase>,
    build_id: i64,
    outcome: i32,
) -> Result<()> {
    sqlx::query(r#"UPDATE ugs_db.IssueBuilds SET (Outcome) = (?) WHERE Id = ?"#)
        .bind(outcome)
        .bind(build_id)
        .execute(&mut *(*sql_connection))
        .await?;
    Ok(())
}

// Private Functions:

fn get_project_like_string(project: Option<&str>) -> String {
    format!(
        "%{}%",
        match project {
            None => String::new(),
            Some(project_string) => get_project_stream(project_string),
        }
    )
}

async fn try_insert_and_get_project(
    sql_connection: &mut Connection<UGSDatabase>,
    project: &str,
) -> Result<i64> {
    let mut transaction = sql_connection.begin().await?;

    sqlx::query(r#"INSERT IGNORE INTO ugs_db.Projects (Name) VALUES (?)"#)
        .bind(project)
        .execute(&mut transaction)
        .await?;

    let id_result =
        sqlx::query_scalar::<_, i64>(r#"SELECT Id FROM ugs_db.Projects WHERE Name = ?"#)
            .bind(project)
            .fetch_one(&mut transaction)
            .await?;

    transaction.commit().await?;

    Ok(id_result)
}

fn get_project_stream(project: &str) -> String {
    use lazy_static::lazy_static;
    use regex::Regex;

    // Get first two fragments of the p4 path.  If it doesn't work, just return back the project.
    lazy_static! {
        static ref STREAM_PATTERN: Regex =
            Regex::new(r#"(//[a-zA-Z0-9\.\-_]{1,}/[a-zA-Z0-9\.\-_]{1,})"#).unwrap();
    }
    let stream_match = STREAM_PATTERN.captures(project);
    match stream_match {
        Some(captures) => captures[1].to_string(),
        None => String::from(project),
    }
}

fn matches_wildcard(wildcard: &str, project: &str) -> bool {
    wildcard.ends_with("...") && project.starts_with(&wildcard[0..wildcard.len() - 4])
}

fn normalize_user_name(user_name: &str) -> String {
    user_name.to_uppercase()
}
