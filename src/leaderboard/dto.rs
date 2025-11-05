/// Data Transfer Objects for Leaderboard
use serde::{Deserialize, Serialize};

/// Leaderboard reset frequency
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum StatisticResetFrequency {
    Hourly,
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

/// Leaderboard entry for a player
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PlayerLeaderboardEntry {
    pub user_name: String,
    #[serde(rename = "UserID")]
    pub user_id: String,
    pub position: i32,
    pub stat_value: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<PlayerProfile>,
}

/// Player profile information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PlayerProfile {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub banned_until: Option<String>,
}

/// Leaderboard result response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetLeaderboardResult {
    pub leaderboard: Vec<PlayerLeaderboardEntry>,
    pub next_reset: Option<String>,
    pub version: i32,
}

/// Leaderboard configuration from title data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Leaderboard {
    pub statistic_name: String,
    pub name: String,
    pub value_name: String,
    pub frequency: StatisticResetFrequency,
    pub rank_rewards: Vec<RankReward>,
}

/// Rank-based rewards configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct RankReward {
    pub rank: String,
    pub items_to_grant: Vec<ItemOrCurrency>,
}

/// Item or currency reward
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ItemOrCurrency {
    #[serde(skip_serializing_if = "Option::is_none", rename = "Type")]
    pub item_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub catalog: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "CurrencyID")]
    pub currency_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "ItemID")]
    pub item_id: Option<String>,
}

/// User's leaderboard rewards to claim
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct UserLeaderboardRewards {
    pub position: i32,
    pub items_to_grant: Vec<ItemOrCurrency>,
}

/// Request to get leaderboard
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetLeaderboardRequest {
    #[serde(rename = "TitleID")]
    pub title_id: String,
    pub build_key: String,
    pub function_name: String,
    pub web_app_link: Option<String>,
    #[serde(rename = "UserID")]
    pub user_id: String,
    pub client_session_ticket: String,
    #[serde(rename = "LeaderboardID")]
    pub leaderboard_id: String,
}

/// Request to claim tournament reward
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ClaimTournamentRewardRequest {
    #[serde(rename = "TitleID")]
    pub title_id: String,
    pub build_key: String,
    pub function_name: String,
    pub web_app_link: Option<String>,
    #[serde(rename = "UserID")]
    pub user_id: String,
    pub client_session_ticket: String,
    pub statistic_name: String,
}

/// User's leaderboard data stored in user data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct UserLeaderboardData {
    pub pending_reward_version: i32,
    pub position: Option<i32>,
    pub stat_value: Option<i32>,
}

/// Request to update a player's statistic
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct UpdateStatisticRequest {
    #[serde(rename = "TitleID")]
    pub title_id: String,
    pub build_key: String,
    #[serde(rename = "UserID")]
    pub user_id: String,
    pub client_session_ticket: String,
    pub statistic_name: String,
    pub value: i32,
}
