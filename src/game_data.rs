use super::LotusApp; // NEW: Import the main app state

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

/// The main event struct, holding the scenario and its options.
/// --- MODIFIED: Now holds a Vec instead of a fixed array ---
#[derive(Debug, Clone)]
pub struct EventData {
    pub title: String,
    pub description: String,
    pub options: Vec<EventOption>,
}

/// This function will be called to generate a new event
/// based on the player's state.
/// --- MODIFIED: Now takes `&LotusApp` to check player stats ---
pub fn generate_event(app: &LotusApp) -> EventData {
    // --- This is your Event Card Matrix lookup ---
    // You can use a `match` statement on `app.player_tier` or `app.player_petal`
    // to return different events.
    // For now, we'll return a rich example.

    let tier_name = ["D", "C", "B", "A", "A+"]
        .get(app.player_tier)
        .cloned()
        .unwrap_or("?");

    let title = format!(
        "Event on Tier {} (Petal {})",
        tier_name, app.player_petal
    );
    let description = "A rival colleague is publicly praised for an idea that was *yours*. \
        Your boss, a Party member, is beaming. This could affect your promotion."
        .to_string();

    // 1. Start with the standard options
    let mut options = vec![
        EventOption {
            text: "A: Say nothing. (Maintain Harmony)".to_string(),
            outcome: EventOutcome {
                scs_change: 5,
                guanxi_party_change: -1, // You look weak to your boss
                ..Default::default()
            },
        },
        EventOption {
            text: "B: Publicly confront your colleague.".to_string(),
            outcome: EventOutcome {
                scs_change: -50, // Causing a scene
                guanxi_network_change: -1,
                guanxi_party_change: -1,
                ..Default::default()
            },
        },
        EventOption {
            text: "C: Privately report them to HR.".to_string(),
            outcome: EventOutcome {
                career_level_change: 0, // Nothing happens
                ..Default::default()
            },
        },
    ];

    // 2. Conditionally add Guanxi-based options
    if app.guanxi_network > 0 {
        options.push(EventOption {
            text: "D: (Use 1 Network) Get friends to 'jokingly' back you up."
                .to_string(),
            outcome: EventOutcome {
                guanxi_network_change: -1, // Spend the token
                career_level_change: 1,    // It works!
                scs_change: -5,            // A bit disruptive
                ..Default::default()
            },
        });
    }

    if app.guanxi_party > 0 {
        options.push(EventOption {
            text: "E: (Use 1 Party) Mention it to your boss later.".to_string(),
            outcome: EventOutcome {
                guanxi_party_change: -1, // Spend the token
                career_level_change: 1,  // He "corrects" the record
                ..Default::default()
            },
        });
    }

    // 3. Return the complete event
    EventData {
        title,
        description,
        options,
    }
}

