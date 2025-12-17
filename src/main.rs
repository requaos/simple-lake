// Declare our new modules
mod app;
mod converter;
mod game_data;
mod lotus_widget;

use crate::game_data::EventData;
use eframe::egui;
use std::fs;
use std::collections::{VecDeque, HashMap};

// --- Floating Text Animation ---
pub struct FloatingText {
    pub text: String,
    pub pos: egui::Pos2,
    pub color: egui::Color32,
    pub age: f32, // In seconds
}

// Define the main application state
pub struct LotusApp {
    // The in-memory database of all possible events
    event_database: Vec<EventData>,
    // Pre-computed index for fast event lookups
    event_index: HashMap<(usize, usize), (Vec<usize>, Vec<usize>)>,

    // Player State
    player_tier: usize,
    player_petal: usize,
    social_credit_score: i32,
    finances: i32,
    career_level: u32,
    guanxi_family: u32,
    guanxi_network: u32,
    guanxi_party: u32,
    player_age: u32,   // NEW: Player's age
    life_stage: usize, // NEW: Current life stage (1-4)

    // Game Board config
    num_petals_per_tier: usize,
    num_tiers: usize,

    // UI State
    current_event: Option<EventData>,
    last_event_result: Option<String>,
    floating_texts: VecDeque<FloatingText>,
    history: Vec<String>,
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
            viewport: egui::ViewportBuilder::default()
                .with_inner_size(egui::vec2(800.0, 800.0))
                .with_app_id("lotus_game"),
            ..Default::default()
        };

        // Load the event database from the JSON file
        let event_json = fs::read_to_string("src/events.json")
            .expect("Failed to read events.json. Did you run `cargo run -- --convert` first?");

        let event_database: Vec<EventData> = serde_json::from_str(&event_json)
            .expect("Failed to parse events.json. Check file format.");

        // --- Pre-compute the event index ---
        let mut event_index = HashMap::new();
        for (i, event) in event_database.iter().enumerate() {
            for tier in event.min_tier..=event.max_tier {
                let (tier_specific, generic) = event_index.entry((event.life_stage, tier)).or_insert_with(|| (Vec::new(), Vec::new()));
                if event.is_generic {
                    generic.push(i);
                } else {
                    tier_specific.push(i);
                }
            }
        }

        // eframe::run_native returns an eframe::Result, so we map the error
        // to anyhow::Error to match our main function's return type.
        eframe::run_native(
            "Lotus Game Board",
            options,
            Box::new(move |cc| {
                let mut visuals = egui::Visuals::dark();
                visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(20, 20, 25); // Deep dark background
                visuals.widgets.active.bg_fill = egui::Color32::from_rgb(200, 50, 50); // Red accents
                visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(218, 165, 32); // Gold for hovered
                cc.egui_ctx.set_visuals(visuals);

                Ok(Box::new(LotusApp {
                    event_database,
                    event_index,
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
                    player_age: 18,  // NEW: Initialize age
                    life_stage: 1,   // NEW: Initialize life stage
                    floating_texts: VecDeque::new(),
                    history: Vec::new(),
                }))
            }),
        )
        .map_err(|e| anyhow::anyhow!("eframe error: {}", e))
    }
}