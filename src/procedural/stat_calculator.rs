use super::library::{StatProfile, Severity};
use rand::Rng;

pub fn calculate_stats(
    base_stats: &StatProfile,
    player_tier: usize,
    severity: Severity,
    rng: &mut impl Rng,
) -> StatProfile {
    let tier_multiplier = (player_tier + 1) as f32 * 1.5;

    let severity_multiplier = match severity {
        Severity::Low => 0.5,
        Severity::Medium => 1.0,
        Severity::High => 2.0,
    };

    let random_variance = rng.gen_range(0.8..=1.2);

    let multiplier = tier_multiplier * severity_multiplier * random_variance;

    StatProfile {
        scs_change: (base_stats.scs_change as f32 * multiplier) as i32,
        finance_change: (base_stats.finance_change as f32 * multiplier) as i32,
        career_level_change: (base_stats.career_level_change as f32 * multiplier) as i32,
        guanxi_family_change: (base_stats.guanxi_family_change as f32 * multiplier) as i32,
        guanxi_network_change: (base_stats.guanxi_network_change as f32 * multiplier) as i32,
        guanxi_party_change: (base_stats.guanxi_party_change as f32 * multiplier) as i32,
    }
}

pub fn calculate_failure_stats(success_stats: &StatProfile) -> StatProfile {
    StatProfile {
        scs_change: -success_stats.scs_change * 3 / 2,
        finance_change: -success_stats.finance_change * 3 / 2,
        career_level_change: -success_stats.career_level_change * 3 / 2,
        guanxi_family_change: -success_stats.guanxi_family_change * 3 / 2,
        guanxi_network_change: -success_stats.guanxi_network_change * 3 / 2,
        guanxi_party_change: -success_stats.guanxi_party_change * 3 / 2,
    }
}
