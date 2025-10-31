use super::LotusApp;
use rand::seq::SliceRandom; // Used for randomly picking an event
use serde::Deserialize; // --- NEW: To read from JSON
use std::collections::HashMap; // --- NEW: For requirements

// --- Core Data Structures ---

/// Defines the stat changes for making a choice.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)] // Makes serde use Default::default() for missing fields
pub struct EventOutcome {
    pub scs_change: i32,
    pub finance_change: i32,
    pub career_level_change: i32,
    pub guanxi_family_change: i32,
    pub guanxi_network_change: i32,
    pub guanxi_party_change: i32,
}

/// A single choice in an event, pairing text with its outcome.
#[derive(Debug, Clone, Deserialize)]
pub struct EventOption {
    pub text: String,
    pub outcome: EventOutcome,
    /// This field holds the *requirements* to see this option.
    /// Example: `{"guanxi_party": 1, "career_level": 3}`
    #[serde(default)] // Will be an empty map if `requirements` is missing in JSON
    pub requirements: HashMap<String, u32>,
}

/// The main event struct, holding all data for a modal window.
#[derive(Debug, Clone, Deserialize)]
pub struct EventData {
    pub title: String,
    pub description: String,
    pub options: Vec<EventOption>, // A list of all possible options
    // --- NEW: Fields for filtering ---
    pub min_tier: usize,
    pub max_tier: usize,
    pub is_generic: bool,
    // We can add `life_stage` here later
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
            // We can add "finances" or "scs" here later
            _ => 0, // Unknown requirement, fail safe
        };

        if player_value < required_value {
            return false; // Player does not meet this requirement
        }
    }
    true // Player meets all requirements
}

/// This function is called by app.rs to get a new event.
/// It selects an event from the in-memory database.
pub fn generate_event(player_state: &LotusApp) -> EventData {
    let mut rng = rand::thread_rng();

    // 1. Try to find a TIER-SPECIFIC event first
    let tier_specific_events: Vec<&EventData> = player_state
        .event_database
        .iter()
        .filter(|event| {
            !event.is_generic
                && event.min_tier <= player_state.player_tier
                && event.max_tier >= player_state.player_tier
            // We could also filter by `life_stage` here
        })
        .collect();

    let chosen_event_template = if let Some(event) = tier_specific_events.choose(&mut rng) {
        // Found a specific event for this tier
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
            // Found a generic event
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
                    outcome: Default::default(),
                    requirements: Default::default(),
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
        // `filter_map` is like `filter` + `map`. We check requirements and clone if met.
        .filter_map(|option| {
            if player_meets_requirements(player_state, &option.requirements) {
                Some(option.clone()) // Clone the option so we can return it
            } else {
                None // This option is not available to the player
            }
        })
        .collect();

    // 5. Return the final event with only the available options
    EventData {
        title: chosen_event_template.title.clone(),
        description: chosen_event_template.description.clone(),
        options: available_options,
        // The rest of the fields don't matter for the modal
        min_tier: 0,
        max_tier: 0,
        is_generic: false,
    }
}

