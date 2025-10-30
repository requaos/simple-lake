// This file contains the data structures for game events and 
// the logic for generating them.

/// The stat changes that result from choosing an event option.
/// All fields are deltas (changes), not absolute values.
#[derive(Debug, Clone, Default)]
pub struct EventOutcome {
    pub scs_change: i32,
    pub finance_change: i32,
    pub career_level_change: i32,
    pub guanxi_family_change: i32,
    pub guanxi_network_change: i32,
    pub guanxi_party_change: i32,
    // We could also add a `next_event_id: Option<String>` for branching narratives
}

/// A single choice a player can make in an event.
/// It pairs the descriptive text with its game outcome.
#[derive(Debug, Clone)]
pub struct EventOption {
    pub text: String,
    pub outcome: EventOutcome,
}

/// The main event struct, holding the scenario and its four possible options.
#[derive(Debug, Clone)]
pub struct EventData {
    pub title: String,
    pub description: String,
    pub options: [EventOption; 4],
}

/// This function will be called to generate a new event
/// based on the player's state.
/// This is where you will build your event database/logic.
pub fn generate_event(player_tier: usize, player_petal: usize) -> EventData {
    // --- This is your Event Card Matrix lookup ---
    // You can use a `match` statement on `player_tier` or `player_petal`
    // to return different events.
    // For now, we'll return a rich example.

    let tier_name = ["D", "C", "B", "A", "A+"]
        .get(player_tier)
        .cloned()
        .unwrap_or("?");

    // Example Event: The Elder's Fall
    EventData {
        title: format!("Event on Tier {} (Petal {})", tier_name, player_petal),
        description:
            "You are walking to work and see an elderly person fall. \
            A 'Citizen Watch' surveillance camera is clearly visible on the corner."
                .to_string(),
        options: [
            EventOption {
                text: "A: Help the person up.".to_string(),
                outcome: EventOutcome {
                    scs_change: 15,
                    finance_change: -10, // You paid for their bus fare home
                    ..Default::default()
                },
            },
            EventOption {
                text: "B: Ignore them and walk past.".to_string(),
                outcome: EventOutcome {
                    scs_change: -30, // Bystander Apathy
                    ..Default::default()
                },
            },
            EventOption {
                text: "C: Publicly scold a nearby youth, then help.".to_string(),
                outcome: EventOutcome {
                    scs_change: 30, // Public Virtue + Upholding Social Standards
                    guanxi_network_change: -1, // The youth's parent is your colleague
                    ..Default::default()
                },
            },
            EventOption {
                text: "D: Call the authorities to handle it.".to_string(),
                outcome: EventOutcome {
                    scs_change: 5, // Correct, but cold
                    ..Default::default()
                },
            },
        ],
    }
}

