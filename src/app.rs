use super::game_data::{generate_event, EventOutcome};
use super::lotus_widget::LotusWidget;
use super::LotusApp;
use eframe::egui::{self, vec2, Align2, Color32, RichText, Window};
use rand::Rng;

// --- MODIFIED: Renamed constants to reflect their purpose ---
const TIER_D_MAX: i32 = 199; // Tier D is <= 199
const TIER_C_MAX: i32 = 399; // Tier C is 200 - 399
const TIER_B_MAX: i32 = 749; // Tier B is 400 - 749
const TIER_A_MAX: i32 = 999; // Tier A is 750 - 999
                             // Tier A+ is anything > 999

impl LotusApp {
    /// Returns true if the petal is one of the SCS review spaces
    fn is_review_petal(&self, petal_index: usize) -> bool {
        petal_index == 0 || petal_index == 4 || petal_index == 8
    }

    /// Safely applies all stat changes from an EventOutcome
    fn apply_outcome(&mut self, outcome: &EventOutcome) {
        self.social_credit_score += outcome.scs_change;
        self.finances += outcome.finance_change;

        // Use saturating_add for u32 values to prevent overflow/underflow
        self.career_level = self
            .career_level
            .saturating_add_signed(outcome.career_level_change);
        self.guanxi_family = self
            .guanxi_family
            .saturating_add_signed(outcome.guanxi_family_change);
        self.guanxi_network = self
            .guanxi_network
            .saturating_add_signed(outcome.guanxi_network_change);
        self.guanxi_party = self
            .guanxi_party
            .saturating_add_signed(outcome.guanxi_party_change);
    }

    /// Checks the player's SCS and updates their tier if needed.
    /// Returns true if the tier changed.
    fn update_player_tier_from_scs(&mut self) -> bool {
        // --- MODIFIED: Corrected tier logic ---
        let new_tier = if self.social_credit_score <= TIER_D_MAX {
            0 // Tier D
        } else if self.social_credit_score <= TIER_C_MAX {
            1 // Tier C
        } else if self.social_credit_score <= TIER_B_MAX {
            2 // Tier B
        } else if self.social_credit_score <= TIER_A_MAX {
            3 // Tier A
        } else {
            4 // Tier A+
        };
        // --- END MODIFICATION ---

        if new_tier != self.player_tier {
            self.player_tier = new_tier;
            true // Tier changed
        } else {
            false // Tier did not change
        }
    }
}

impl eframe::App for LotusApp {
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

                        // If we moved to a new petal AND it's not a review space, trigger an event
                        if !self.is_review_petal(self.player_petal) {
                            self.current_event = Some(generate_event(self));
                        }
                        // Clear last event result on move
                        self.last_event_result = None;
                    }
                    if ui.button("Move Clockwise").clicked() {
                        self.player_petal = (self.player_petal + 1) % self.num_petals_per_tier;

                        // If we moved to a new petal AND it's not a review space, trigger an event
                        if !self.is_review_petal(self.player_petal) {
                            self.current_event = Some(generate_event(self));
                        }
                        // Clear last event result on move
                        self.last_event_result = None;
                    }
                });
            });

            // --- Last Event Result ---
            if let Some(result_text) = &self.last_event_result {
                if !result_text.is_empty() {
                    ui.add_space(5.0);
                    // Show event result in a distinct color
                    ui.label(
                        RichText::new(result_text)
                            .color(Color32::from_rgb(200, 200, 100))
                            .strong(),
                    );
                    ui.add_space(5.0);
                }
            }

            // --- Player Stats Panel ---
            ui.add_enabled_ui(!event_is_open, |ui: &mut egui::Ui| {
                egui::Frame::group(ui.style()).show(ui, |ui| {
                    ui.label(RichText::new("Player Stats").strong());
                    ui.horizontal_wrapped(|ui| {
                        ui.label(format!("SCS: {}", self.social_credit_score));
                        ui.label(format!("Finances (¥): {}", self.finances));
                        ui.label(format!("Career: Lvl {}", self.career_level));
                    });
                    ui.horizontal_wrapped(|ui| {
                        ui.label(format!("Guanxi (Family): {}", self.guanxi_family));
                        ui.label(format!("Guanxi (Network): {}", self.guanxi_network));
                        ui.label(format!("Guanxi (Party): {}", self.guanxi_party));
                    });
                });
            });

            // --- Status/Review UI ---
            if !event_is_open {
                // If the player is on an "SCS Review" space, check for tier change
                if self.is_review_petal(self.player_petal) {
                    ui.add_space(5.0);
                    ui.label(RichText::new("You are on an 'SCS Review' space!").strong());
                    ui.label("Your SCS is re-evaluated...");

                    if self.update_player_tier_from_scs() {
                        ui.label(
                            RichText::new(format!(
                                "Your Tier has changed to {}!",
                                self.player_tier
                            ))
                            .color(Color32::RED)
                            .strong(),
                        );
                    } else {
                        ui.label("Your Tier remains unchanged.");
                    }
                    ui.add_space(5.0);
                }

                // Helper to map tier index to SCS name
                let tier_name = [
                    "D (Blacklisted)",
                    "C (Warning)",
                    "B (Standard)",
                    "A (Trusted)",
                    "A+ (Exemplary)",
                ]
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
                    player_total_index,
                ))
            };
            ui.add_enabled(!event_is_open, draw_lotus_widget);

            // --- Event Window (Modal) ---
            if let Some(event) = self.current_event.clone() {
                Window::new(RichText::new(&event.title).strong())
                    .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
                    .collapsible(false)
                    .resizable(false)
                    .show(ctx, |ui| {
                        ui.set_max_width(300.0);
                        ui.add(egui::Label::new(&event.description).wrap());
                        ui.separator();

                        // --- Dynamic buttons with risk logic ---
                        ui.vertical_centered_justified(|ui| {
                            for option in event.options.iter() {
                                if ui.button(&option.text).clicked() {
                                    // --- MODIFIED: Use thread_rng() to get a valid Rng implementation ---
                                    let mut rng = rand::thread_rng();
                                    
                                    if option.risk_chance > 0
                                        && rng.gen_range(1..=100) <= option.risk_chance
                                    {
                                        // --- FAILURE ---
                                        if let Some(outcome) = &option.failure_outcome {
                                            self.apply_outcome(outcome);
                                        }
                                        self.last_event_result =
                                            Some(option.failure_result.clone());
                                    } else {
                                        // --- SUCCESS ---
                                        self.apply_outcome(&option.success_outcome);
                                        self.last_event_result =
                                            Some(option.success_result.clone());
                                    }

                                    self.current_event = None; // Close window
                                }
                            }
                        });
                        // --- END MODIFICATION ---
                    });
            }
        });
    }
}

