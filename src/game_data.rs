use super::LotusApp;
use rand::prelude::IndexedRandom; // MODIFIED: Use trait suggested by compiler
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
    
    // NEW: Risk and multiple outcomes
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
/// It selects an event from the in-memory database.
pub fn generate_event(player_state: &LotusApp) -> EventData {
    // MODIFIED: Use new rand API
    let mut rng = rand::rng();

    // 1. Try to find a TIER-SPECIFIC event first
    let tier_specific_events: Vec<&EventData> = player_state
        .event_database
        .iter()
        .filter(|event| {
            !event.is_generic
                && event.min_tier <= player_state.player_tier
                && event.max_tier >= player_state.player_tier
        })
        .collect();

    let chosen_event_template = if let Some(event) = tier_specific_events.choose(&mut rng) {
        *event
    } else {
        // 2. If no specific event, find a GENERIC event
        let generic_events: Vec<&EventData> = player_state
            .event_database
            .iter()
            .filter(|event| {
                event.is_generic
                    && event.min_tier <= player_state.player_tier
                    && event.max_tier >= player_state.player_tier
            })
            .collect();

        if let Some(event) = generic_events.choose(&mut rng) {
            *event
        } else {
            // 3. Fallback: If no events are found for this tier, create a default panic event
            return EventData {
                title: "No Event Found!".to_string(),
                description: format!(
                    "Error: No events found for player tier {}. Please check events.json.",
                    player_state.player_tier
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
            };
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
    }
}

