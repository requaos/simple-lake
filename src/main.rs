use eframe::NativeOptions;
use egui::vec2;

// Declare our new modules
pub mod app;
pub mod game_data;
pub mod lotus_widget;

// Use the public structs from our modules
use game_data::EventData;

/// Main application state
/// This struct definition stays in main.rs,
/// as it's the central state for the whole application.
pub struct LotusApp {
    // Game Board State
    player_tier: usize,
    player_petal: usize,
    num_petals_per_tier: usize,
    num_tiers: usize,

    // Modal Event State
    current_event: Option<EventData>,

    // --- NEW: Player Stats ---
    social_credit_score: i32,
    finances: i32,
    career_level: u32,
    guanxi_family: u32,
    guanxi_network: u32,
    guanxi_party: u32,
}

/// Main function to run the app
fn main() -> eframe::Result<()> {
    let options = NativeOptions {
        // Set a default window size, but it's now fully resizable
        viewport: egui::ViewportBuilder::default().with_inner_size(vec2(800.0, 800.0)),
        ..Default::default()
    };

    eframe::run_native(
        "Lotus Game Board",
        options,
        Box::new(|_cc| Ok(Box::<LotusApp>::default())),
    )
}

