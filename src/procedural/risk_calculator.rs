use std::collections::HashMap;

pub fn calculate_risk(
    base_risk: u8,
    risk_modifier: i8,
    requirements: &HashMap<String, u32>,
    player_state: &PlayerStats,
) -> u8 {
    let mut risk = base_risk as i32;

    // Adjust based on requirement gaps
    for (stat_name, &required_value) in requirements {
        let player_value = match stat_name.as_str() {
            "guanxi_family" => player_state.guanxi_family,
            "guanxi_network" => player_state.guanxi_network,
            "guanxi_party" => player_state.guanxi_party,
            "career_level" => player_state.career_level,
            _ => 0,
        };

        let gap = required_value.saturating_sub(player_value);
        risk += (gap * 5) as i32;
    }

    // Apply choice archetype modifier
    risk += risk_modifier as i32;

    // Clamp to 0-95 range
    risk.clamp(0, 95) as u8
}

// Helper struct to pass player stats
pub struct PlayerStats {
    pub guanxi_family: u32,
    pub guanxi_network: u32,
    pub guanxi_party: u32,
    pub career_level: u32,
}
