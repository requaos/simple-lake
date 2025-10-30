/// A struct to hold event data
/// Add `pub` so fields are visible to other modules
#[derive(Debug, Clone)]
pub struct EventData {
    pub title: String,
    pub description: String,
    pub options: [String; 4],
}

/// This function will be called to generate a new event
/// based on the player's state.
pub fn generate_event(player_tier: usize, player_petal: usize) -> EventData {
    // This is where you can build your event database.
    // You can use a big `match` statement on (player_tier, player_petal),
    // or eventually load from a file.

    let tier_name = ["D", "C", "B", "A", "A+"]
        .get(player_tier)
        .cloned()
        .unwrap_or("?");

    // Example of varying event by tier
    let description = if player_tier == 0 {
        "You are in re-education. A guard offers you extra rations for \
         reporting on another player. What do you do?"
            .to_string()
    } else {
        "You are walking to work and see an elderly person fall. \
         A 'Citizen Watch' surveillance camera is clearly visible on the corner."
            .to_string()
    };

    EventData {
        title: format!("Event on Tier {} (Petal {})", tier_name, player_petal),
        description,
        options: [
            "A: [Placeholder Option 1]".to_string(),
            "B: [Placeholder Option 2]".to_string(),
            "C: [Placeholder Option 3]".to_string(),
            "D: [Placeholder Option 4]".to_string(),
        ],
    }
}
