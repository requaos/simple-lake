use eframe::{egui, NativeOptions};
use egui::vec2;
use game_data::EventData; // Import the EventData struct

// --- Module declarations ---
// This tells main.rs that these files exist.
mod app;
mod game_data;
mod lotus_widget;

// --- Main Application Struct ---
/// This struct now holds the application's persistent state.
/// All the transient UI state is in `app.rs`.
pub struct LotusApp {
    player_tier: usize,
    player_petal: usize,
    num_petals_per_tier: usize,
    num_tiers: usize,

    // --- Core Game State ---
    social_credit_score: i32,
    finances: i32,
    career_level: u32,
    guanxi_family: u32,
    guanxi_network: u32,
    guanxi_party: u32,

    // --- Modal/Event State ---
    current_event: Option<EventData>,

    // --- IN-MEMORY DATABASE ---
    /// This vector holds all events loaded from events.json
    event_database: Vec<EventData>,
}

impl LotusApp {
    /// Create a new instance of the application
    pub fn new(event_database: Vec<EventData>) -> Self {
        Self {
            player_tier: 2, // Start on Tier 2 (SCS Tier 'B')
            player_petal: 1, // Start on petal 1 (not the review space)
            num_petals_per_tier: 13,
            num_tiers: 5,
            current_event: None, // No event window is open at the start

            // --- Initialize Player Stats ---
            social_credit_score: 500, // Tier B
            finances: 1000,
            career_level: 1,
            guanxi_family: 2,
            guanxi_network: 1,
            guanxi_party: 1, // Start with one party token for testing

            // --- Store the loaded event database ---
            event_database,
        }
    }
}

/// Main function to run the app
fn main() -> eframe::Result<()> {
    // --- Load the Event Database from file ---
    // This includes the events.json file directly into the binary
    const EVENTS_JSON: &str = include_str!("events.json");
    // Parse the JSON into our Rust structs
    let event_database: Vec<EventData> =
        serde_json::from_str(EVENTS_JSON).expect("Failed to load events.json. Check syntax.");
    println!("Loaded {} events from events.json", event_database.len());

    let options = NativeOptions {
        // Set a default window size, but it's now fully resizable
        viewport: egui::ViewportBuilder::default().with_inner_size(vec2(800.0, 800.0)),
        ..Default::default()
    };

    eframe::run_native(
        "Lotus Game Board",
        options,
        Box::new(|_cc| {
            // Create the app state and pass the loaded database
            Ok(Box::new(LotusApp::new(event_database)))
        }),
    )
}

