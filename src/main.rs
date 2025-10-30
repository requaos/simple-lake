use eframe::NativeOptions;
use egui::vec2;

// Declare our new modules
mod app;
mod game_data;
mod lotus_widget;

// Use the public structs from our modules
use app::LotusApp;
use game_data::EventData;

/// Main application state
/// This struct definition stays in main.rs,
/// but its implementation is in app.rs.
/// We add `pub` so app.rs can see the fields.
pub struct LotusApp {
    pub player_tier: usize,
    pub player_petal: usize,
    pub num_petals_per_tier: usize,
    pub num_tiers: usize,
    pub current_event: Option<EventData>, // Holds the active event
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
