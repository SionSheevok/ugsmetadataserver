use chrono::serde::{ts_seconds, ts_seconds_option};
use rocket::serde::{Deserialize, Serialize};
use serde_repr::{Serialize_repr, Deserialize_repr};
use rocket_db_pools::sqlx::encode::IsNull;
use rocket_db_pools::sqlx::error::BoxDynError;
use rocket_db_pools::sqlx::{Decode, Encode, FromRow, MySql, Type};
use sqlx::mysql::{MySqlTypeInfo, MySqlValueRef};
use std::fmt;

type DateTime = chrono::DateTime<chrono::Utc>;

#[derive(Debug, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(i32)]
pub enum BuildResult {
    Starting = 0,
    Failure = 1,
    Warning = 2,
    Success = 3,
    Skipped = 4,
}

impl fmt::Display for BuildResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Type<MySql> for BuildResult {
    fn type_info() -> MySqlTypeInfo {
        <str as Type<MySql>>::type_info()
    }

    fn compatible(ty: &MySqlTypeInfo) -> bool {
        <str as Type<MySql>>::compatible(ty)
    }
}

impl Encode<'_, MySql> for BuildResult {
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> IsNull {
        <&str as Encode<MySql>>::encode(&*(self.to_string()), buf)
    }
}

impl Decode<'_, MySql> for BuildResult {
    fn decode(value: MySqlValueRef<'_>) -> Result<Self, BoxDynError> {
        let map_op = |from_str: &str| -> Self {
            match from_str {
                "Starting" => Self::Starting,
                "Failure" => Self::Failure,
                "Warning" => Self::Warning,
                "Success" => Self::Success,
                "Skipped" => Self::Skipped,
                &_ => unimplemented!(),
            }
        };
        <&str as Decode<MySql>>::decode(value).map(map_op)
    }
}

#[derive(Serialize, Deserialize, FromRow)]
#[sqlx(rename_all = "PascalCase")]
#[serde(rename_all = "PascalCase")]
pub struct BuildData {
    #[serde(skip_deserializing)]
    pub id: i64,
    pub change_number: i32,
    pub build_type: String,
    pub result: BuildResult,
    pub url: String,
    pub project: String,
    pub archive_path: String,
}

#[derive(Serialize, Deserialize, FromRow)]
#[sqlx(rename_all = "PascalCase")]
#[serde(rename_all = "PascalCase")]
pub struct CommentData {
    #[serde(skip_deserializing)]
    pub id: i64,
    pub change_number: i32,
    pub user_name: String,
    pub text: String,
    pub project: String,
}

#[derive(Debug, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(i32)]
pub enum EventType {
    Syncing = 0,

    // Reviews
    Compiles = 1,
    DoesNotCompile = 2,
    Good = 3,
    Bad = 4,
    Unknown = 5,

    // Starred builds
    Starred = 6,
    Unstarred = 7,

    // Investigating events
    Investigating = 8,
    Resolved = 9,
}

impl fmt::Display for EventType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Type<MySql> for EventType {
    fn type_info() -> MySqlTypeInfo {
        <str as Type<MySql>>::type_info()
    }

    fn compatible(ty: &MySqlTypeInfo) -> bool {
        <str as Type<MySql>>::compatible(ty)
    }
}

impl Encode<'_, MySql> for EventType {
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> IsNull {
        <&str as Encode<MySql>>::encode(&*(self.to_string()), buf)
    }
}

impl Decode<'_, MySql> for EventType {
    fn decode(value: MySqlValueRef<'_>) -> Result<Self, BoxDynError> {
        let map_op = |from_str: &str| -> Self {
            match from_str {
                "Syncing" => Self::Syncing,
                "Compiles" => Self::Compiles,
                "DoesNotCompile" => Self::DoesNotCompile,
                "Good" => Self::Good,
                "Bad" => Self::Bad,
                "Unknown" => Self::Unknown,
                "Starred" => Self::Starred,
                "Unstarred" => Self::Unstarred,
                "Investigating" => Self::Investigating,
                "Resolved" => Self::Resolved,
                &_ => unimplemented!(),
            }
        };
        <&str as Decode<MySql>>::decode(value).map(map_op)
    }
}

#[derive(Serialize, Deserialize, FromRow)]
#[sqlx(rename_all = "PascalCase")]
#[serde(rename_all = "PascalCase")]
pub struct EventData {
    #[serde(skip_deserializing)]
    pub id: i64,
    pub change: i32,
    pub user_name: String,
    #[serde(rename = "Type")]
    pub event_type: EventType,
    pub project: String,
}

#[derive(Debug, PartialEq, Eq, Deserialize_repr)]
#[repr(i32)]
pub enum ReviewVerdict {
    Unknown = 0,
    Good = 1,
    Bad = 2,
    Mixed = 3,
}

impl fmt::Display for ReviewVerdict {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Type<MySql> for ReviewVerdict {
    fn type_info() -> MySqlTypeInfo {
        <str as Type<MySql>>::type_info()
    }

    fn compatible(ty: &MySqlTypeInfo) -> bool {
        <str as Type<MySql>>::compatible(ty)
    }
}

impl Encode<'_, MySql> for ReviewVerdict {
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> IsNull {
        <&str as Encode<MySql>>::encode(&*(self.to_string()), buf)
    }
}

impl Decode<'_, MySql> for ReviewVerdict {
    fn decode(value: MySqlValueRef<'_>) -> Result<Self, BoxDynError> {
        let map_op = |from_str: &str| -> Self {
            match from_str {
                "Unknown" => Self::Unknown,
                "Good" => Self::Good,
                "Bad" => Self::Bad,
                "Mixed" => Self::Mixed,
                &_ => unimplemented!(),
            }
        };
        <&str as Decode<MySql>>::decode(value).map(map_op)
    }
}

#[derive(Deserialize, FromRow)]
#[sqlx(rename_all = "PascalCase")]
#[serde(rename_all = "PascalCase")]
pub struct EventSummary {
    pub change_number: i32,
    pub verdict: ReviewVerdict,
    pub sync_events: Vec<EventData>,
    pub reviews: Vec<EventData>,
    pub current_users: Vec<String>,
    pub last_star_review: EventData,
    pub builds: Vec<BuildData>,
    pub comments: Vec<CommentData>,
}

#[derive(Deserialize, FromRow)]
#[sqlx(rename_all = "PascalCase")]
#[serde(rename_all = "PascalCase")]
pub struct IssueWatcherData {
    pub user_name: String,
}

#[derive(Serialize, Deserialize, FromRow)]
#[sqlx(rename_all = "PascalCase")]
#[serde(rename_all = "PascalCase")]
pub struct IssueBuildData {
    #[serde(skip_deserializing)]
    pub id: i64,
    pub stream: String,
    pub change: i32,
    pub job_name: String,
    pub job_url: String,
    pub job_step_name: String,
    pub job_step_url: String,
    pub error_url: String,
    pub outcome: i32,
}

#[derive(Deserialize, FromRow)]
#[sqlx(rename_all = "PascalCase")]
#[serde(rename_all = "PascalCase")]
pub struct IssueBuildUpdateData {
    pub outcome: i32,
}

#[derive(Serialize, Deserialize, FromRow)]
#[sqlx(rename_all = "PascalCase")]
#[serde(rename_all = "PascalCase")]
pub struct IssueDiagnosticData {
    pub build_id: Option<i64>,
    pub message: String,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
#[sqlx(rename_all = "PascalCase")]
#[serde(rename_all = "PascalCase")]
pub struct IssueData {
    #[serde(skip_deserializing)]
    pub id: i64,
    #[serde(with = "ts_seconds")]
    pub created_at: DateTime,
    #[serde(with = "ts_seconds")]
    pub retrieved_at: DateTime,
    pub project: String,
    pub summary: String,
    pub owner: String,
    pub nominated_by: String,
    #[serde(with = "ts_seconds_option")]
    pub acknowledged_at: Option<DateTime>,
    pub fix_change: i32,
    #[serde(with = "ts_seconds_option")]
    pub resolved_at: Option<DateTime>,
    pub notify: bool,
}

#[derive(Debug, Deserialize, FromRow)]
#[sqlx(rename_all = "PascalCase")]
#[serde(rename_all = "PascalCase")]
pub struct IssueUpdateData {
    pub summary: String,
    pub owner: String,
    pub nominated_by: String,
    pub acknowledged: Option<bool>,
    pub fix_change: Option<i32>,
    pub resolved: Option<bool>,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct LatestData {
    pub last_event_id: i64,
    pub last_comment_id: i64,
    pub last_build_id: i64,
}

#[derive(Debug, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(i32)]
pub enum TelemetryErrorType {
    Crash = 0,
}

impl fmt::Display for TelemetryErrorType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Type<MySql> for TelemetryErrorType {
    fn type_info() -> MySqlTypeInfo {
        <str as Type<MySql>>::type_info()
    }

    fn compatible(ty: &MySqlTypeInfo) -> bool {
        <str as Type<MySql>>::compatible(ty)
    }
}

impl Encode<'_, MySql> for TelemetryErrorType {
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> IsNull {
        <&str as Encode<MySql>>::encode(&*(self.to_string()), buf)
    }
}

impl Decode<'_, MySql> for TelemetryErrorType {
    fn decode(value: MySqlValueRef<'_>) -> Result<Self, BoxDynError> {
        let map_op = |from_str: &str| -> Self {
            match from_str {
                "Crash" => Self::Crash,
                &_ => unimplemented!(),
            }
        };
        <&str as Decode<MySql>>::decode(value).map(map_op)
    }
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
#[sqlx(rename_all = "PascalCase")]
#[serde(rename_all = "PascalCase")]
pub struct TelemetryErrorData {
    #[serde(skip_deserializing)]
    pub id: i64,
    #[serde(rename = "Type")]
    pub error_type: TelemetryErrorType,
    pub text: String,
    pub user_name: String,
    pub project: Option<String>,
    #[serde(with = "ts_seconds")]
    pub timestamp: DateTime,
    pub version: String,
    pub ip_address: String,
}

#[derive(Debug, Deserialize, FromRow)]
#[sqlx(rename_all = "PascalCase")]
#[serde(rename_all = "PascalCase")]
pub struct TelemetryTimingData {
    pub action: String,
    pub result: String,
    pub user_name: String,
    pub project: String,
    #[serde(with = "ts_seconds")]
    pub timestamp: DateTime,
    pub duration: f32,
}
