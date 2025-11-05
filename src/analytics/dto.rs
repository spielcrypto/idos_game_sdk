/// Data Transfer Objects for Analytics
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsEvent {
    pub event_name: String,
    pub properties: HashMap<String, serde_json::Value>,
    pub timestamp: i64,
    pub session_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStartEvent {
    pub platform: String,
    pub device_info: DeviceInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub platform: String,
    pub browser: Option<String>,
    pub os: Option<String>,
    pub screen_resolution: Option<String>,
    pub language: Option<String>,
}
