// Declare our new modules
mod app;
mod converter;
mod game_data;
mod lotus_widget;

use crate::game_data::EventData;
// Removed unused HashMap import
use std::fs;

// Define the main application state
pub struct LotusApp {
    // The in-memory database of all possible events
    event_database: Vec<EventData>,

    // Player State
    player_tier: usize,
    player_petal: usize,
    social_credit_score: i32,
    finances: i32,
    career_level: u32,
    guanxi_family: u32,
    guanxi_network: u32,
    guanxi_party: u32,

    // Game Board config
    num_petals_per_tier: usize,
    num_tiers: usize,

    // UI State
    current_event: Option<EventData>,
    last_event_result: Option<String>,
}

fn main() -> anyhow::Result<()> {
    // 1. Check command line arguments
    let args: Vec<String> = std::env::args().collect();

    if args.contains(&"--convert".to_string()) {
        // 2. If --convert is found, run the converter logic
        println!("Running event data converter...");
        converter::run_converter()?; // This will propogate any errors
        println!("Successfully generated 'src/events.json' from CSVs. Exiting.");
        Ok(()) // Exit successfully
    } else {
        // 3. Otherwise, run the game
        println!("Starting game...");
        let options = eframe::NativeOptions {
            viewport: eframe::egui::ViewportBuilder::default()
                .with_inner_size(eframe::egui::vec2(800.0, 800.0)),
            ..Default::default()
        };

        // Load the event database from the JSON file
        let event_json = fs::read_to_string("src/events.json")
            .expect("Failed to read events.json. Did you run `cargo run -- --convert` first?");

        let event_database: Vec<EventData> = serde_json::from_str(&event_json)
            .expect("Failed to parse events.json. Check file format.");

        // eframe::run_native returns an eframe::Result, so we map the error
        // to anyhow::Error to match our main function's return type.
        eframe::run_native(
            "Lotus Game Board",
            options,
            Box::new(move |_cc| {
                Ok(Box::new(LotusApp {
                    event_database, // Pass the loaded data
                    player_tier: 2,
                    player_petal: 1,
                    num_petals_per_tier: 13,
                    num_tiers: 5,
                    social_credit_score: 550, // Start in Tier B
                    finances: 1000,
                    career_level: 1,
                    guanxi_family: 1,
                    guanxi_network: 1,
                    guanxi_party: 0,
                    current_event: None,
                    last_event_result: None,
                }))
            }),
        )
        .map_err(|e| anyhow::anyhow!("eframe error: {}", e))
    }
}

