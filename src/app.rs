use eframe::{egui, App};
use egui::{vec2, Align2, Window};

// --- MODIFIED: Removed unused `EventData` import ---
use super::game_data::generate_event;
use super::lotus_widget::LotusWidget;
use super::LotusApp;

/// Implementation of the application's default state
impl Default for LotusApp {
    fn default() -> Self {
        Self {
            player_tier: 2, // Start on Tier 2 (SCS Tier 'B')
            player_petal: 1, // Start on petal 1 (not the review space)
            num_petals_per_tier: 8,
            num_tiers: 5,
            current_event: None, // No event window is open at the start
        }
    }
}

/// Implementation of the main application loop (UI)
impl App for LotusApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Check if a modal window is open.
            let event_is_open = self.current_event.is_some();

            // --- Top Controls ---
            // --- MODIFIED: Wrap the horizontal layout in add_enabled ---
            ui.add_enabled(!event_is_open, |ui: &mut egui::Ui| {
                ui.horizontal(|ui| {
                    if ui.button("Exit Application").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }

                    ui.label("Click the buttons to move the player token.");

                    if ui.button("Move Counter-Clockwise").clicked() {
                        self.player_petal = (self.player_petal + self.num_petals_per_tier - 1)
                            % self.num_petals_per_tier;
                        // If we moved to a new petal AND it's not the review space, trigger an event
                        if self.player_petal != 0 {
                            // Use the generate_event function from our game_data module
                            self.current_event = Some(generate_event(
                                self.player_tier,
                                self.player_petal,
                            ));
                        }
                    }
                }) // --- MODIFIED: Removed semicolon to return the Response ---
            });
            // --- END MODIFICATION ---

            // --- Status/Review UI ---
            // This entire section is hidden if an event is open
            if !event_is_open {
                // If the player is on the "SCS Review" space (petal 0), show tier change buttons
                if self.player_petal == 0 {
                    ui.add_space(5.0);
                    ui.label(egui::RichText::new("You are on an 'SCS Review' space!").strong());
                    ui.label("Your SCS is re-evaluated...");
                    ui.horizontal(|ui| {
                        if ui.button("SCS: Go Up a Tier").clicked() {
                            // Use .min() to clamp at the max tier
                            self.player_tier = (self.player_tier + 1).min(self.num_tiers - 1);
                        }
                        if ui.button("SCS: Go Down a Tier").clicked() {
                            // .saturating_sub() prevents underflow (going below 0)
                            self.player_tier = self.player_tier.saturating_sub(1);
                        }
                    });
                    ui.add_space(5.0);
                }

                // Helper to map tier index to SCS name
                let tier_name = ["D (Blacklisted)", "C (Warning)", "B (Standard)", "A (Trusted)", "A+ (Exemplary)"]
                    .get(self.player_tier)
                    .cloned()
                    .unwrap_or("?");

                ui.label(format!(
                    "Player is on Tier {} (SCS: {}), Petal {}",
                    self.player_tier, tier_name, self.player_petal
                ));
                ui.add_space(10.0);
            }

            // --- Game Board Widget ---
            // We draw the board *before* the modal, so it's in the background.
            let draw_lotus_widget = |ui: &mut egui::Ui| {
                let player_total_index =
                    self.player_tier * self.num_petals_per_tier + self.player_petal;

                ui.add(LotusWidget::new(
                    self.num_tiers,
                    self.num_petals_per_tier,
                    player_total_index, // Pass the calculated total index
                )) // Removed semicolon to return the Response
            };

            // If an event is open, draw the widget disabled (dimmed).
            // Otherwise, draw it enabled.
            ui.add_enabled(!event_is_open, draw_lotus_widget);

            // --- Event Window (Modal) ---
            // Drawn last, so it's on top of everything else.
            // We clone the event to avoid borrow checker issues.
            if let Some(event) = self.current_event.clone() {
                Window::new(egui::RichText::new(&event.title).strong())
                    .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0)) // Center the window
                    .collapsible(false)
                    .resizable(false)
                    .show(ctx, |ui| {
                        ui.set_max_width(300.0); // Constrain window width

                        ui.add(egui::Label::new(&event.description).wrap());

                        ui.separator();

                        // Show the four options
                        ui.vertical_centered_justified(|ui| {
                            if ui.button(&event.options[0]).clicked() {
                                println!("Player chose Option 1");
                                self.current_event = None; // Close window
                            }
                            if ui.button(&event.options[1]).clicked() {
                                println!("Player chose Option 2");
                                self.current_event = None; // Close window
                            }
                            if ui.button(&event.options[2]).clicked() {
                                println!("Player chose Option 3");
                                self.current_event = None; // Close window
                            }
                            if ui.button(&event.options[3]).clicked() {
                                println!("Player chose Option 4");
                                self.current_event = None; // Close window
                            }
                        });
                    });
            }
        });
    }
}

