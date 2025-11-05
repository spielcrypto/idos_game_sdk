/// Leaderboard handler for competitive rankings
use super::dto::*;
use crate::{IdosClient, IdosError, IdosResult};
use bevy::prelude::Resource;

#[derive(Resource, Clone)]
pub struct LeaderboardHandler {
    client: IdosClient,
    user_id: Option<String>,
    session_ticket: Option<String>,
}

impl LeaderboardHandler {
    pub fn new(client: IdosClient) -> Self {
        Self {
            client,
            user_id: None,
            session_ticket: None,
        }
    }

    /// Set user authentication info (call after login)
    pub fn set_auth(&mut self, user_id: String, session_ticket: String) {
        self.user_id = Some(user_id);
        self.session_ticket = Some(session_ticket);
    }

    /// Clear authentication info (call on logout)
    pub fn clear_auth(&mut self) {
        self.user_id = None;
        self.session_ticket = None;
    }

    fn get_user_id(&self) -> IdosResult<String> {
        self.user_id
            .clone()
            .ok_or_else(|| IdosError::Auth("User not logged in".to_string()))
    }

    fn get_session_ticket(&self) -> IdosResult<String> {
        self.session_ticket
            .clone()
            .ok_or_else(|| IdosError::Auth("No session ticket available".to_string()))
    }

    /// Get leaderboard rankings
    /// Returns top players and the requesting player's position
    pub async fn get_leaderboard(&self, leaderboard_id: &str) -> IdosResult<GetLeaderboardResult> {
        let request = GetLeaderboardRequest {
            title_id: self.client.game_id().to_string(),
            build_key: String::new(),
            function_name: "GetLeaderboard".to_string(),
            web_app_link: None,
            user_id: self.get_user_id()?,
            client_session_ticket: self.get_session_ticket()?,
            leaderboard_id: leaderboard_id.to_string(),
        };

        let endpoint = "user-data-system/GetLeaderboard";
        self.client.post(endpoint, &request).await
    }

    /// Claim tournament rewards for a statistic
    /// Call this when a player has pending rewards from a leaderboard
    pub async fn claim_tournament_reward(
        &self,
        statistic_name: &str,
    ) -> IdosResult<UserLeaderboardRewards> {
        let request = ClaimTournamentRewardRequest {
            title_id: self.client.game_id().to_string(),
            build_key: String::new(),
            function_name: "ClaimTournamentReward".to_string(),
            web_app_link: None,
            user_id: self.get_user_id()?,
            client_session_ticket: self.get_session_ticket()?,
            statistic_name: statistic_name.to_string(),
        };

        let endpoint = "tournament/ClaimTournamentReward";
        self.client.post(endpoint, &request).await
    }

    /// Update player's statistic value (score)
    /// Note: In production, score updates usually happen on server-side to prevent cheating
    pub async fn update_statistic(&self, statistic_name: &str, value: i32) -> IdosResult<String> {
        let request = UpdateStatisticRequest {
            title_id: self.client.game_id().to_string(),
            build_key: String::new(),
            user_id: self.get_user_id()?,
            client_session_ticket: self.get_session_ticket()?,
            statistic_name: statistic_name.to_string(),
            value,
        };

        let endpoint = "statistics/update";
        self.client.post(endpoint, &request).await
    }

    /// Check if player has pending rewards for any leaderboard
    pub fn has_pending_rewards(&self, leaderboard_data: &UserLeaderboardData) -> bool {
        leaderboard_data.pending_reward_version > 0
    }

    /// Calculate player's reward based on rank
    pub fn get_reward_for_rank(
        &self,
        rank_rewards: &[RankReward],
        player_position: i32,
    ) -> Option<Vec<ItemOrCurrency>> {
        for rank_reward in rank_rewards {
            if self.is_position_in_rank(&rank_reward.rank, player_position) {
                return Some(rank_reward.items_to_grant.clone());
            }
        }
        None
    }

    /// Check if a position matches a rank string (e.g., "1", "2-5", "6-10")
    fn is_position_in_rank(&self, rank: &str, position: i32) -> bool {
        if rank.contains('-') {
            // Range format: "2-5"
            let parts: Vec<&str> = rank.split('-').collect();
            if parts.len() == 2 {
                if let (Ok(start), Ok(end)) = (parts[0].parse::<i32>(), parts[1].parse::<i32>()) {
                    return position >= start && position <= end;
                }
            }
        } else {
            // Single rank: "1"
            if let Ok(rank_num) = rank.parse::<i32>() {
                return position == rank_num;
            }
        }
        false
    }
}
