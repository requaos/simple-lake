use eframe::{egui, App};
use egui::{vec2, Align2, Window};

// Use the items we've moved to other files
use super::game_data::{generate_event, EventOutcome};
use super::lotus_widget::LotusWidget;
use super::LotusApp;

// --- Logic block for applying game changes ---
impl LotusApp {
    /// Takes an EventOutcome and safely applies all stat changes to the player.
    pub fn apply_outcome(&mut self, outcome: &EventOutcome) {
        // Apply simple i32 deltas
        self.social_credit_score += outcome.scs_change;
        self.finances += outcome.finance_change;

        // Apply deltas to u32 stats, ensuring they don't go below 0
        self.career_level = (self.career_level as i32 + outcome.career_level_change).max(0) as u32;
        self.guanxi_family =
            (self.guanxi_family as i32 + outcome.guanxi_family_change).max(0) as u32;
        self.guanxi_network =
            (self.guanxi_network as i32 + outcome.guanxi_network_change).max(0) as u32;
        self.guanxi_party =
            (self.guanxi_party as i32 + outcome.guanxi_party_change).max(0) as u32;

        // Tier check is purposefully NOT here.
        // It will only be checked on the "SCS Review" petal for tension.
    }

    /// --- NEW: Helper to check if a petal is a review space ---
    fn is_review_petal(&self, petal_index: usize) -> bool {
        // With 13 petals, let's use 0, 4, and 8 as review spaces
        petal_index == 0 || petal_index == 4 || petal_index == 8
    }

    /// --- Logic to automatically adjust player tier based on SCS ---
    pub fn update_player_tier_from_scs(&mut self) {
        let new_tier = match self.social_credit_score {
            1000.. => 4,    // Tier A+ (Exemplary)
            750..=999 => 3, // Tier A (Trusted)
            400..=749 => 2, // Tier B (Standard)
            200..=399 => 1, // Tier C (Warning)
            _ => 0,         // Tier D (Blacklisted) (anything below 200)
        };

        if new_tier != self.player_tier {
            println!(
                "SCS Review: Player tier changed from {} to {}",
                self.player_tier, new_tier
            );
            self.player_tier = new_tier;
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
            ui.add_enabled_ui(!event_is_open, |ui: &mut egui::Ui| {
                ui.horizontal(|ui| {
                    if ui.button("Exit Application").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }

                    ui.label("Click the buttons to move the player token.");

                    if ui.button("Move Counter-Clockwise").clicked() {
                        self.player_petal = (self.player_petal + self.num_petals_per_tier - 1)
                            % self.num_petals_per_tier;
                        // If we moved to a new petal AND it's not the review space, trigger an event
                        if !self.is_review_petal(self.player_petal) {
                            self.current_event = Some(generate_event(self));
                        }
                    }
                    if ui.button("Move Clockwise").clicked() {
                        self.player_petal = (self.player_petal + 1) % self.num_petals_per_tier;
                        // If we moved to a new petal AND it's not the review space, trigger an event
                        if !self.is_review_petal(self.player_petal) {
                            self.current_event = Some(generate_event(self));
                        }
                    }
                });
            });
            // --- END Top Controls ---

            // --- Player Stats Panel ---
            ui.add_enabled_ui(!event_is_open, |ui| {
                egui::Frame::group(ui.style()).show(ui, |ui| {
                    ui.label(egui::RichText::new("Player Stats").strong());
                    ui.horizontal(|ui| {
                        ui.label(format!("SCS: {}", self.social_credit_score));
                        ui.label(format!("Finances (¥): {}", self.finances));
                        ui.label(format!("Career: Lvl {}", self.career_level));
                    });
                    ui.horizontal(|ui| {
                        ui.label(format!("Guanxi (Family): {}", self.guanxi_family));
                        ui.label(format!("Guanxi (Network): {}", self.guanxi_network));
                        ui.label(format!("Guanxi (Party): {}", self.guanxi_party));
                    });
                });
            });
            ui.add_space(5.0);
            // --- END Stats Panel ---

            // --- Status/Review UI ---
            // This entire section is hidden if an event is open
            if !event_is_open {
                // If the player is on an "SCS Review" space
                if self.is_review_petal(self.player_petal) {
                    ui.add_space(5.0);
                    ui.label(egui::RichText::new("You are on an 'SCS Review' space!").strong());
                    ui.label("Your SCS is re-evaluated...");

                    // Automatically update tier
                    self.update_player_tier_from_scs();

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
            ui.add_enabled(!event_is_open, draw_lotus_widget);

            // --- Event Window (Modal) ---
            if let Some(event) = self.current_event.clone() {
                Window::new(egui::RichText::new(&event.title).strong())
                    .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0)) // Center the window
                    .collapsible(false)
                    .resizable(false)
                    .show(ctx, |ui| {
                        ui.set_max_width(300.0); // Constrain window width
                        ui.add(egui::Label::new(&event.description).wrap());
                        ui.separator();

                        // Iterate over the Vec of options
                        ui.vertical_centered_justified(|ui| {
                            for (index, option) in event.options.iter().enumerate() {
                                if ui.button(&option.text).clicked() {
                                    println!("Player chose Option {}", index + 1);
                                    self.apply_outcome(&option.outcome);
                                    self.current_event = None; // Close window
                                }
                            }
                        });
                    });
            }
        });
    }
}

