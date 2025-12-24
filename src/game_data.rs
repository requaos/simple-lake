use super::LotusApp;
use crate::procedural;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// --- Core Data Structures ---

/// Defines the stat changes for making a choice.
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
#[serde(default)]
pub struct EventOutcome {
    pub scs_change: i32,
    pub finance_change: i32,
    pub career_level_change: i32,
    pub guanxi_family_change: i32,
    pub guanxi_network_change: i32,
    pub guanxi_party_change: i32,
}

/// A single choice in an event, pairing text with its outcome.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EventOption {
    pub text: String,
    #[serde(default)]
    pub requirements: HashMap<String, u32>,

    // Risk and multiple outcomes
    #[serde(default)]
    pub risk_chance: u8, // Chance of failure (0-100)

    pub success_outcome: EventOutcome,
    pub success_result: String, // Text to show on success

    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_outcome: Option<EventOutcome>,
    #[serde(default)]
    #[serde(skip_serializing_if = "String::is_empty")]
    pub failure_result: String, // Text to show on failure
}

/// The main event struct, holding all data for a modal window.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EventData {
    pub title: String,
    pub description: String,
    pub options: Vec<EventOption>, // A list of all possible options
    pub min_tier: usize,
    pub max_tier: usize,
    pub is_generic: bool,
    pub life_stage: usize, // NEW: Which life stage this event belongs to

    // Procedural generation metadata
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub procedural_id: Option<String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub procedural_domain: Option<String>,
}

// --- Main Event Generation Function ---

/// Checks if the player meets the requirements for a specific option.
fn player_meets_requirements(player_state: &LotusApp, requirements: &HashMap<String, u32>) -> bool {
    for (key, &required_value) in requirements {
        let player_value = match key.as_str() {
            "guanxi_family" => player_state.guanxi_family,
            "guanxi_network" => player_state.guanxi_network,
            "guanxi_party" => player_state.guanxi_party,
            "career_level" => player_state.career_level,
            _ => 0,
        };

        if player_value < required_value {
            return false;
        }
    }
    true
}

/// This function is called by app.rs to get a new event.
/// It first attempts procedural generation, then falls back to handcrafted events.
pub fn generate_event(player_state: &LotusApp) -> EventData {
    use rand::prelude::IndexedRandom;
    let mut rng = rand::rng();
    let current_tier = player_state.player_tier;
    let current_stage = player_state.life_stage;

    // Attempt procedural generation first
    if let Some(procedural_event) = procedural::generate_procedural_event(player_state, &mut rng) {
        return procedural_event;
    }

    // Fallback to handcrafted events
    log::info!("=== FALLING BACK TO HANDCRAFTED EVENTS ===");
    log::info!("  Reason: Procedural generation returned None");
    log::info!("  Player state: tier={}, life_stage={}", current_tier, current_stage);

    let mut potential_events: Vec<usize> = Vec::new();

    // 1. Try to find a non-generic (tier-specific) event for the current stage
    if let Some((tier_specific, _)) = player_state.event_index.get(&(current_stage, current_tier)) {
        log::debug!("  Found {} tier-specific events for stage={}, tier={}", tier_specific.len(), current_stage, current_tier);
        potential_events.extend(tier_specific);
    }

    let chosen_event_template: &EventData = if let Some(&event_index) =
        potential_events.choose(&mut rng)
    {
        log::info!("✓ Selected tier-specific handcrafted event: '{}'", player_state.event_database[event_index].title);
        &player_state.event_database[event_index]
    } else {
        // 2. If none, find a generic event for the current stage
        log::debug!("  No tier-specific events, trying generic events");
        if let Some((_, generic)) = player_state.event_index.get(&(current_stage, current_tier)) {
            log::debug!("  Found {} generic events for stage={}, tier={}", generic.len(), current_stage, current_tier);
            potential_events.extend(generic);
        }
        if let Some(&event_index) = potential_events.choose(&mut rng) {
            log::info!("✓ Selected generic handcrafted event: '{}'", player_state.event_database[event_index].title);
            &player_state.event_database[event_index]
        } else {
            // 3. Fallback: find *any* generic event from a past life stage
            log::debug!("  No generic events for current stage, trying past life stages");
            for stage in (1..current_stage).rev() {
                if let Some((_, generic)) = player_state.event_index.get(&(stage, current_tier)) {
                    log::debug!("  Found {} generic events from past stage={}", generic.len(), stage);
                    potential_events.extend(generic);
                }
            }
            if let Some(&event_index) = potential_events.choose(&mut rng) {
                log::info!("✓ Selected past life stage handcrafted event: '{}'", player_state.event_database[event_index].title);
                &player_state.event_database[event_index]
            } else {
                // 4. Absolute fallback
                log::error!("!!! NO EVENTS FOUND !!!");
                log::error!("  No handcrafted events available for tier={}, life_stage={}", current_tier, current_stage);
                log::error!("  Returning error event");
                return EventData {
                    title: "No Event Found!".to_string(),
                    description: format!(
                        "Error: No events found for player tier {} and life stage {}. Please check events.json.",
                        player_state.player_tier, player_state.life_stage
                    ),
                    options: vec![EventOption {
                        text: "Continue".to_string(),
                        requirements: Default::default(),
                        risk_chance: 0,
                        success_outcome: Default::default(),
                        success_result: "".to_string(),
                        failure_outcome: None,
                        failure_result: "".to_string(),
                    }],
                    min_tier: 0,
                    max_tier: 99,
                    is_generic: true,
                    life_stage: 0,
                    procedural_id: None,
                    procedural_domain: None,
                };
            }
        }
    };

    // 4. We have an event template. Now, filter its options based on player state.
    let available_options: Vec<EventOption> = chosen_event_template
        .options
        .iter()
        .filter_map(|option| {
            if player_meets_requirements(player_state, &option.requirements) {
                Some(option.clone())
            } else {
                None
            }
        })
        .collect();

    // 5. Return the final event with only the available options
    EventData {
        title: chosen_event_template.title.clone(),
        description: chosen_event_template.description.clone(),
        options: available_options,
        min_tier: 0,
        max_tier: 0,
        is_generic: false,
        life_stage: chosen_event_template.life_stage,
        procedural_id: None,
        procedural_domain: None,
    }
}
