use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnisetteData {
    pub machine_id: String,
    pub one_time_password: String,
    pub local_user_id: String,
    pub routing_info: u64,
    pub device_unique_identifier: String,
    pub device_serial_number: String,
    pub device_description: String,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub date: DateTime<Utc>,
    pub locale: String,
    pub time_zone: String,
}
