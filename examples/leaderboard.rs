/// Example demonstrating Leaderboard functionality
/// Competitive rankings, tournaments, and rewards
///
/// Run with: cargo run --example leaderboard --features leaderboard

#[cfg(feature = "leaderboard")]
use idos_game_sdk::leaderboard::{
    dto::{StatisticResetFrequency, UserLeaderboardData},
    handler::LeaderboardHandler,
};

#[cfg(feature = "leaderboard")]
use idos_game_sdk::{IdosClient, IdosConfig, IdosResult};

#[cfg(feature = "leaderboard")]
#[tokio::main]
async fn main() -> IdosResult<()> {
    println!("🏆 Leaderboard Example\n");

    // Initialize client
    let config = IdosConfig {
        api_key: "your_api_key".to_string(),
        game_id: "your_game_id".to_string(),
        ..Default::default()
    };

    let client = IdosClient::new(config);
    let mut leaderboard = LeaderboardHandler::new(client);

    // Set authentication (normally from login response)
    leaderboard.set_auth("user123".to_string(), "session_ticket_xyz".to_string());

    println!("✅ Leaderboard handler initialized\n");

    // Example 1: Get leaderboard rankings
    println!("📊 Example 1: Get Leaderboard Rankings");
    println!("──────────────────────────────────────");

    let leaderboard_id = "high_score_weekly";
    match leaderboard.get_leaderboard(leaderboard_id).await {
        Ok(result) => {
            println!("✅ Leaderboard '{}' retrieved", leaderboard_id);
            println!("   Total entries: {}", result.leaderboard.len());
            println!("   Version: {}", result.version);
            println!("   Next reset: {:?}", result.next_reset);

            // Show top 5
            println!("\n   Top 5 Players:");
            for (i, entry) in result.leaderboard.iter().take(5).enumerate() {
                println!(
                    "   {}. {} - {} points (Position: {})",
                    i + 1,
                    entry.user_name,
                    entry.stat_value,
                    entry.position
                );
            }
            println!();
        }
        Err(e) => println!("❌ Failed: {}\n", e),
    }

    // Example 2: Update player's score
    println!("🎮 Example 2: Update Player Score");
    println!("─────────────────────────────────");
    println!("Note: In production, updates should happen server-side to prevent cheating!");

    match leaderboard
        .update_statistic("high_score_weekly", 1500)
        .await
    {
        Ok(response) => {
            println!("✅ Score updated successfully");
            println!("   New score: 1500 points");
            println!("   Response: {}\n", response);
        }
        Err(e) => println!("❌ Failed: {}\n", e),
    }

    // Example 3: Check for pending rewards
    println!("🎁 Example 3: Check for Pending Rewards");
    println!("───────────────────────────────────────");

    let user_data = UserLeaderboardData {
        pending_reward_version: 1,
        position: Some(3),
        stat_value: Some(1500),
    };

    if leaderboard.has_pending_rewards(&user_data) {
        println!("✅ Player has pending rewards!");
        println!("   Position: {:?}", user_data.position);
        println!("   Reward version: {}", user_data.pending_reward_version);
        println!("   → Call claim_tournament_reward() to collect\n");
    } else {
        println!("ℹ️  No pending rewards\n");
    }

    // Example 4: Claim tournament rewards
    println!("💰 Example 4: Claim Tournament Rewards");
    println!("──────────────────────────────────────");

    match leaderboard
        .claim_tournament_reward("high_score_weekly")
        .await
    {
        Ok(rewards) => {
            println!("✅ Rewards claimed successfully!");
            println!("   Player position: {}", rewards.position);
            println!("   Rewards granted:");

            for (i, item) in rewards.items_to_grant.iter().enumerate() {
                if let Some(currency_id) = &item.currency_id {
                    println!(
                        "   {}. {} x{} (currency)",
                        i + 1,
                        currency_id,
                        item.amount.unwrap_or(0)
                    );
                } else if let Some(item_id) = &item.item_id {
                    println!(
                        "   {}. {} x{} (item)",
                        i + 1,
                        item_id,
                        item.amount.unwrap_or(0)
                    );
                }
            }
            println!();
        }
        Err(e) => println!("❌ Failed: {}\n", e),
    }

    // Example 5: Calculate rewards for rank
    println!("🏅 Example 5: Reward Calculation");
    println!("────────────────────────────────");

    use idos_game_sdk::leaderboard::dto::{ItemOrCurrency, RankReward};

    let rank_rewards = vec![
        RankReward {
            rank: "1".to_string(),
            items_to_grant: vec![ItemOrCurrency {
                item_type: Some("Currency".to_string()),
                catalog: None,
                amount: Some(1000),
                image_path: None,
                name: Some("Gold".to_string()),
                currency_id: Some("GOLD".to_string()),
                item_id: None,
            }],
        },
        RankReward {
            rank: "2-5".to_string(),
            items_to_grant: vec![ItemOrCurrency {
                item_type: Some("Currency".to_string()),
                catalog: None,
                amount: Some(500),
                image_path: None,
                name: Some("Gold".to_string()),
                currency_id: Some("GOLD".to_string()),
                item_id: None,
            }],
        },
        RankReward {
            rank: "6-10".to_string(),
            items_to_grant: vec![ItemOrCurrency {
                item_type: Some("Currency".to_string()),
                catalog: None,
                amount: Some(100),
                image_path: None,
                name: Some("Gold".to_string()),
                currency_id: Some("GOLD".to_string()),
                item_id: None,
            }],
        },
    ];

    let test_positions = vec![1, 3, 7, 15];
    for position in test_positions {
        if let Some(rewards) = leaderboard.get_reward_for_rank(&rank_rewards, position) {
            println!("   Position {}: Gets {} rewards", position, rewards.len());
            for reward in rewards {
                if let Some(currency_id) = &reward.currency_id {
                    println!("      → {} x{}", currency_id, reward.amount.unwrap_or(0));
                }
            }
        } else {
            println!("   Position {}: No rewards", position);
        }
    }

    println!("\n✨ Leaderboard examples complete!");
    println!("\n📚 Available Operations:");
    println!("   • get_leaderboard() - Get rankings and player position");
    println!("   • update_statistic() - Update player's score");
    println!("   • claim_tournament_reward() - Claim leaderboard rewards");
    println!("   • has_pending_rewards() - Check for unclaimed rewards");
    println!("   • get_reward_for_rank() - Calculate rewards for position");

    println!("\n🔄 Leaderboard Frequencies:");
    println!("   • Hourly - Resets every hour");
    println!("   • Daily - Resets every day");
    println!("   • Weekly - Resets every week");
    println!("   • Monthly - Resets every month");
    println!("   • Yearly - Resets every year");

    Ok(())
}

// Fallback main for non-leaderboard builds
#[cfg(not(feature = "leaderboard"))]
fn main() {
    println!("❌ This example requires the 'leaderboard' feature.");
    println!("Run with: cargo run --example leaderboard --features leaderboard");
}
