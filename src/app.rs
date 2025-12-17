use super::game_data::{generate_event, EventOutcome};
use super::lotus_widget::LotusWidget;
use super::{FloatingText, LotusApp};
use eframe::egui::{self, vec2, Align2, Color32, RichText, Window, Area, Order, Id, Pos2, Rect};
use rand::Rng;

impl LotusApp {
    // Add a queue for floating text animations
    fn add_floating_text(&mut self, text: String, pos: Pos2, color: Color32) {
        self.floating_texts.push_back(FloatingText {
            text,
            pos,
            color,
            age: 0.0,
        });
    }
}

// --- Tier Definitions ---
const TIER_D_MAX: i32 = 199; // Tier D is <= 199
const TIER_C_MAX: i32 = 399; // Tier C is 200 - 399
const TIER_B_MAX: i32 = 749; // Tier B is 400 - 749
const TIER_A_MAX: i32 = 999; // Tier A is 750 - 999
                             // Tier A+ is anything > 999

// --- NEW: Life Stage Definitions ---
const AGE_STAGE_2: u32 = 26; // Early Career (26-40)
const AGE_STAGE_3: u32 = 41; // Mid-Career (41-55)
const AGE_STAGE_4: u32 = 56; // Seniority (56+)

impl LotusApp {
    /// Returns true if the petal is one of the SCS review spaces
    fn is_review_petal(&self, petal_index: usize) -> bool {
        petal_index == 0 || petal_index == 4 || petal_index == 8
    }

    /// Safely applies all stat changes from an EventOutcome
    fn apply_outcome(&mut self, outcome: &EventOutcome, ui_rect: Rect) {
        // --- Floating Text ---
        let base_pos = ui_rect.center_top();
        if outcome.scs_change != 0 {
            let text = format!("{} SCS", outcome.scs_change);
            let color = if outcome.scs_change > 0 { Color32::GREEN } else { Color32::RED };
            self.add_floating_text(text, base_pos, color);
        }
        if outcome.finance_change != 0 {
            let text = format!("{} ¥", outcome.finance_change);
            let color = if outcome.finance_change > 0 { Color32::GOLD } else { Color32::RED };
            self.add_floating_text(text, Pos2::new(base_pos.x + 20.0, base_pos.y), color);
        }
        // ... add more for other stats if desired

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

        if new_tier != self.player_tier {
            self.player_tier = new_tier;
            true // Tier changed
        } else {
            false // Tier did not change
        }
    }

    // --- NEW: Age Progression ---
    /// Increments player age and checks for life stage changes.
    fn age_up(&mut self) {
        self.player_age += 1;
        self.last_event_result = Some(format!("Happy Birthday! You are now {}.", self.player_age));
        self.update_life_stage(); // Check if this new age triggers a new life stage
    }

    /// Updates the player's life stage based on their new age.
    fn update_life_stage(&mut self) {
        let new_stage = if self.player_age >= AGE_STAGE_4 {
            4
        } else if self.player_age >= AGE_STAGE_3 {
            3
        } else if self.player_age >= AGE_STAGE_2 {
            2
        } else {
            1
        };

        if new_stage != self.life_stage {
            self.life_stage = new_stage;
            // Overwrite the birthday message with a more important one
            self.last_event_result = Some(format!(
                "You are {}. You've entered a new Life Stage: {}!",
                self.player_age, self.life_stage
            ));
        }
    }
}

impl eframe::App for LotusApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let event_is_open = self.current_event.is_some();

        // --- Left Stats Panel ---
        let left_panel_response = egui::SidePanel::left("left_panel")
            .resizable(false)
            .default_width(180.0)
            .show(ctx, |ui| {
                ui.add_enabled_ui(!event_is_open, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("Player Status");
                    });
                    ui.separator();
                    ui.label(format!("Age: {}", self.player_age));
                    ui.label(format!("Life Stage: {}", self.life_stage));
                    ui.label(RichText::new(format!("Social Credit: {}", self.social_credit_score)).strong());
                    ui.label(format!("Finances (¥): {}", self.finances));
                    ui.label(format!("Career: Lvl {}", self.career_level));
                });
                ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::Default); // Ensure default cursor
            });

        // --- Right Guanxi Panel ---
        egui::SidePanel::right("right_panel")
            .resizable(false)
            .default_width(180.0)
            .show(ctx, |ui| {
                ui.add_enabled_ui(!event_is_open, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("Guanxi Network");
                    });
                    ui.separator();
                    ui.label(format!("Family: {}", self.guanxi_family));
                    ui.label(format!("Network: {}", self.guanxi_network));
                    ui.label(format!("Party: {}", self.guanxi_party));
                });
            });

        // --- Main Central Panel ---
        egui::CentralPanel::default().show(ctx, |ui| {
            // --- Top Controls ---
            ui.add_enabled_ui(!event_is_open, |ui| {
                ui.horizontal(|ui| {
                    if ui.button("Exit Application").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                    let old_petal = self.player_petal;
                    let mut moved = false;
                    if ui.button("Move Counter-Clockwise").clicked() {
                        self.player_petal = (self.player_petal + self.num_petals_per_tier - 1) % self.num_petals_per_tier;
                        moved = true;
                        if self.player_petal > old_petal { self.age_up(); }
                    }
                    if ui.button("Move Clockwise").clicked() {
                        self.player_petal = (self.player_petal + 1) % self.num_petals_per_tier;
                        moved = true;
                        if self.player_petal < old_petal { self.age_up(); }
                    }
                    if moved {
                        if !self.is_review_petal(self.player_petal) {
                            self.current_event = Some(generate_event(self));
                            self.last_event_result = None;
                        } else {
                            self.current_event = None;
                            if self.player_petal != 0 { self.last_event_result = None; }
                        }
                    }
                });
            });

            // --- Last Event Result & Status ---
            if !event_is_open {
                if let Some(result_text) = &self.last_event_result {
                    if !result_text.is_empty() {
                        ui.label(RichText::new(result_text).color(Color32::from_rgb(200, 200, 100)).strong());
                    }
                }
                if self.is_review_petal(self.player_petal) {
                    ui.label(RichText::new("SCS Review...").strong());
                    if self.update_player_tier_from_scs() {
                        ui.label(RichText::new(format!("Tier changed to {}!", self.player_tier)).color(Color32::RED).strong());
                    } else {
                        ui.label("Tier remains unchanged.");
                    }
                }
            }

            // --- Game Board Widget ---
            ui.centered_and_justified(|ui| {
                let player_total_index = self.player_tier * self.num_petals_per_tier + self.player_petal;
                ui.add(LotusWidget::new(self.num_tiers, self.num_petals_per_tier, player_total_index));
            });
        });

        // --- Event Modal (as an Area) ---
        if let Some(event) = self.current_event.clone() {
            // Darkened overlay
            Area::new(Id::new("event_overlay"))
                .fixed_pos(ctx.screen_rect().min)
                .order(Order::Middle)
                .show(ctx, |ui| {
                    ui.painter().rect_filled(ctx.screen_rect(), 0.0, Color32::from_black_alpha(180));
                });

            // Event Window
            Window::new(RichText::new(&event.title).strong())
                .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.set_max_width(350.0);
                    ui.add(egui::Label::new(&event.description).wrap());
                    ui.separator();
                    ui.vertical_centered_justified(|ui| {
                        for option in event.options.iter() {
                            let button_response = ui.button(&option.text);

                            // --- Predictive Tooltip ---
                            button_response.clone().on_hover_ui(|ui| {
                                let risk_text = if option.risk_chance > 75 { "Very High" }
                                                else if option.risk_chance > 50 { "High" }
                                                else if option.risk_chance > 25 { "Medium" }
                                                else if option.risk_chance > 0 { "Low" }
                                                else { "None" };
                                ui.label(format!("Risk: {} ({}%)", risk_text, option.risk_chance));
                            });

                            if button_response.clicked() {
                                let mut rng = rand::thread_rng();
                                if option.risk_chance > 0 && rng.gen_range(1..=100) <= option.risk_chance {
                                    if let Some(outcome) = &option.failure_outcome { self.apply_outcome(outcome, left_panel_response.response.rect); }
                                    self.last_event_result = Some(option.failure_result.clone());
                                } else {
                                    self.apply_outcome(&option.success_outcome, left_panel_response.response.rect);
                                    self.last_event_result = Some(option.success_result.clone());
                                }
                                self.current_event = None;
                            }
                        }
                    });
                });
        }

        // --- Floating Text System ---
        let delta_time = ctx.input(|i| i.stable_dt);
        self.floating_texts.retain_mut(|ft| {
            ft.age += delta_time;
            ft.pos.y -= delta_time * 30.0; // Move up
            ft.age < 2.0 // Keep for 2 seconds
        });

        Area::new(Id::new("floating_text_area"))
            .fixed_pos(Pos2::ZERO)
            .order(Order::Tooltip)
            .show(ctx, |ui| {
                for ft in &self.floating_texts {
                    let alpha = ((2.0 - ft.age) / 2.0).max(0.0); // Fade out
                    let color = ft.color.linear_multiply(alpha);
                    ui.painter().text(ft.pos, Align2::CENTER_CENTER, &ft.text, egui::FontId::proportional(16.0), color);
                }
            });
    }
}