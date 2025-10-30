use eframe::{egui, App, NativeOptions};
use egui::{
    vec2, Align2, Color32, FontId, Pos2, Response, Rgba, Sense, Shape, Stroke, Ui, Vec2, Widget,
};
use std::f32::consts::TAU; // TAU is 2 * PI

/// Main application state
struct LotusApp {
    player_total_index: usize,
    num_petals_per_tier: usize,
    num_tiers: usize,
}

impl Default for LotusApp {
    fn default() -> Self {
        Self {
            player_total_index: 0,
            num_petals_per_tier: 8,
            num_tiers: 5,
        }
    }
}

impl App for LotusApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Lotus Flower Game Board");

            if ui.button("Exit Application").clicked() {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }

            ui.label("Click the buttons to move the player token.");
            ui.add_space(10.0);

            let total_petals = self.num_petals_per_tier * self.num_tiers;

            // --- UI Controls to move the player ---
            ui.horizontal(|ui| {
                if ui.button("Move Counter-Clockwise").clicked() {
                    self.player_total_index =
                        (self.player_total_index + total_petals - 1) % total_petals;
                }
                if ui.button("Move Clockwise").clicked() {
                    self.player_total_index = (self.player_total_index + 1) % total_petals;
                }
            });
            ui.label(format!(
                "Player is on petal index: {}",
                self.player_total_index
            ));
            ui.add_space(50.0);

            // Center the widget
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                // Add our custom lotus widget, passing in the player's target index
                ui.add(LotusWidget::new(
                    self.num_tiers,
                    self.num_petals_per_tier,
                    150.0, // Increased base radius further to fit text
                    self.player_total_index,
                ));
            });

            // Repaint continuously to see animations
            ctx.request_repaint();
        });
    }
}

/// Our custom widget.
struct LotusWidget {
    num_tiers: usize,
    num_petals_per_tier: usize,
    base_radius: f32, // The radius of the *outermost* tier
    player_total_index: usize,
}

impl LotusWidget {
    pub fn new(
        num_tiers: usize,
        num_petals_per_tier: usize,
        base_radius: f32,
        player_total_index: usize,
    ) -> Self {
        Self {
            num_tiers,
            num_petals_per_tier,
            base_radius,
            player_total_index,
        }
    }

    /// Helper function to get the "resting position" on a petal.
    fn get_petal_resting_pos(&self, total_index: usize, center: Pos2) -> Pos2 {
        let tier = total_index / self.num_petals_per_tier;
        let petal = total_index % self.num_petals_per_tier;

        // Calculate this tier's radius (from 1/N to N/N)
        let tier_radius_factor = (tier as f32 + 1.0) / self.num_tiers as f32;
        let tier_radius = self.base_radius * tier_radius_factor;

        // Offset each tier's rotation by half a petal
        let tier_rotation = (tier as f32 * (TAU / self.num_petals_per_tier as f32)) / 2.0;
        let angle = (petal as f32 / self.num_petals_per_tier as f32) * TAU + tier_rotation;

        // Use (sin, -cos) to have index 0 at the top (12 o'clock)
        // Place it 75% of the way out on that tier's radius
        let offset_vec = vec2(angle.sin(), -angle.cos()) * tier_radius * 0.75;
        center + offset_vec
    }

    /// Helper to create a petal shape for drawing or hit-testing
    fn create_petal_shape(
        &self,
        center: Pos2,
        radius: f32, // This is now the radius for the *current tier*
        angle: f32,
        scale: f32,
        fill: Color32,
        stroke: Stroke,
    ) -> Shape {
        let p0 = center;
        let p3 = center;

        // Petal size is relative to the tier's radius
        // --- MODIFIED ---
        let petal_width = radius * 0.8 * scale; // Increased from 0.5 to 0.8
        let petal_length = radius * 1.0 * scale; // Decreased from 1.2 to 1.0
        // --- END MODIFICATION ---

        let cp1_base = vec2(-petal_width, -petal_length);
        let cp2_base = vec2(petal_width, -petal_length);

        // Rotate the control points by the petal's angle
        let p1 = center + rotate_vec(cp1_base, angle);
        let p2 = center + rotate_vec(cp2_base, angle);

        Shape::CubicBezier(egui::epaint::CubicBezierShape {
            points: [p0, p1, p2, p3],
            closed: true,
            fill,
            stroke: stroke.into(),
        })
    }

    /// Helper function to get the text for a specific petal
    /// This is where you would define your game board content
    fn get_petal_text(&self, tier: usize, petal: usize, total_index: usize) -> String {
        // Tier 0 is the innermost (Blacklisted)
        // Tier 4 is the outermost (Exemplary)

        // Special "SCS Review" space on the first petal of each tier
        if petal == 0 {
            return "SCS\nReview".to_string();
        }

        // --- Example Game Logic ---
        match tier {
            0 => "Re-education".to_string(), // Tier D (Blacklisted)
            1 => "Job Warning".to_string(),  // Tier C (Warning)
            2 => format!("Event\n{}", total_index), // Tier B (Standard)
            3 => "Party Banquet".to_string(), // Tier A (Trusted)
            4 => "Honored!".to_string(),     // Tier A+ (Exemplary)
            _ => "??".to_string(),
        }
    }
}

/// Implementation of the `Widget` trait for our `LotusWidget`.
impl Widget for LotusWidget {
    fn ui(self, ui: &mut Ui) -> Response {
        // 1. Allocate space for the widget
        // Accommodate the largest radius
        let desired_size = vec2(self.base_radius * 2.5, self.base_radius * 2.5);
        let (rect, mut response) = ui.allocate_exact_size(desired_size, Sense::hover());

        let center = rect.center();
        let painter = ui.painter();
        let ctx = ui.ctx();

        // Tier colors (Mapped to SCS Tiers)
        let tier_colors = [
            Rgba::from(Color32::from_rgb(80, 80, 80)),      // Tier 0 (D - Blacklisted) - Dark Grey
            Rgba::from(Color32::from_rgb(255, 100, 100)),   // Tier 1 (C - Warning) - Red
            Rgba::from(Color32::from_rgb(255, 180, 105)),   // Tier 2 (B - Standard) - Pink/Orange
            Rgba::from(Color32::from_rgb(105, 200, 255)),   // Tier 3 (A - Trusted) - Blue
            Rgba::from(Color32::from_rgb(255, 220, 100)), // Tier 4 (A+ - Exemplary) - Gold
        ];

        // 2. Iterate and draw each petal for each tier
        // (Reversed loop, draws from back (largest) to front (smallest))
        for tier in (0..self.num_tiers).rev() {
            // Calculate this tier's radius (from 1/N to N/N)
            let tier_radius_factor = (tier as f32 + 1.0) / self.num_tiers as f32;
            let tier_radius = self.base_radius * tier_radius_factor;

            // Offset each tier's rotation by half a petal
            let tier_rotation = (tier as f32 * (TAU / self.num_petals_per_tier as f32)) / 2.0;

            // Get tier color
            let base_color_rgba = tier_colors
                .get(tier)
                .cloned()
                .unwrap_or(Rgba::from(Color32::GRAY));
            let hover_color_rgba = egui::lerp(base_color_rgba..=Rgba::WHITE, 0.4);

            for petal in 0..self.num_petals_per_tier {
                let petal_total_index = tier * self.num_petals_per_tier + petal;
                let petal_id = response.id.with(petal_total_index);
                let angle = (petal as f32 / self.num_petals_per_tier as f32) * TAU + tier_rotation;

                // --- Interaction: ---
                let base_shape = self.create_petal_shape(
                    center,
                    tier_radius, // Use this tier's radius
                    angle,
                    1.0,
                    Color32::TRANSPARENT,
                    Stroke::NONE,
                );
                let hover_rect = base_shape.visual_bounding_rect();
                let petal_response = ui.interact(hover_rect, petal_id, Sense::click_and_drag());
                let is_hovered = petal_response.hovered();
                let is_clicked = petal_response.is_pointer_button_down_on();

                // --- Animation: ---
                let scale_anim = ctx.animate_value_with_time(
                    petal_id.with("scale"),
                    if is_hovered { 1.2 } else { 1.0 },
                    0.1,
                );
                let click_flash = ctx.animate_value_with_time(
                    petal_id.with("click"),
                    if is_clicked { 1.0 } else { 0.0 },
                    0.1,
                );

                // --- Color Logic ---
                let click_color = Rgba::from(Color32::WHITE);
                let hover_progress = (scale_anim - 1.0) / 0.2; // 0.0 to 1.0
                let color_rgba = egui::lerp(base_color_rgba..=hover_color_rgba, hover_progress);
                let color_rgba_with_click = egui::lerp(color_rgba..=click_color, click_flash);
                let final_color: Color32 = color_rgba_with_click.into();

                // --- Drawing: ---
                let petal_shape = self.create_petal_shape(
                    center,
                    tier_radius, // Use this tier's radius
                    angle,
                    scale_anim,
                    final_color,
                    Stroke::new(1.0, Color32::from_black_alpha(60)),
                );

                painter.add(petal_shape);

                // --- NEW: DRAW PETAL TEXT ---
                // We draw this *after* the petal shape, so it's on top
                let petal_text_pos = self.get_petal_resting_pos(petal_total_index, center);
                let text = self.get_petal_text(tier, petal, petal_total_index);

                painter.text(
                    petal_text_pos,
                    Align2::CENTER_CENTER,
                    text,
                    FontId::proportional(12.0),
                    Color32::BLACK,
                );
                // --- END NEW ---

                response |= petal_response;
            }
        }

        // --- 3. Draw the Animated Player Token ---
        // (This is drawn last, so it's always on top, which is correct)
        let target_pos = self.get_petal_resting_pos(self.player_total_index, center);
        let player_anim_id = response.id.with("player_token_pos");

        // Animate X and Y components separately
        let animated_x =
            ctx.animate_value_with_time(player_anim_id.with("x"), target_pos.x, 0.3);
        let animated_y =
            ctx.animate_value_with_time(player_anim_id.with("y"), target_pos.y, 0.3);

        // Combine them back into a Pos2
        let animated_pos = Pos2::new(animated_x, animated_y);

        // Draw the player token
        painter.circle_filled(
            animated_pos,
            10.0,
            Color32::from_rgb(255, 220, 0), // Yellow
        );
        painter.circle_stroke(
            animated_pos,
            10.0,
            Stroke::new(2.0, Color32::from_black_alpha(150)),
        );

        response
    }
}

/// Helper function to rotate a Vec2
fn rotate_vec(v: Vec2, angle: f32) -> Vec2 {
    let (sin, cos) = angle.sin_cos();
    vec2(v.x * cos - v.y * sin, v.x * sin + v.y * cos)
}

/// Main function to run the app
fn main() -> eframe::Result<()> {
    let options = NativeOptions {
        // Increased window size
        viewport: egui::ViewportBuilder::default().with_inner_size(vec2(800.0, 800.0)), // Made window larger
        ..Default::default()
    };

    eframe::run_native(
        "Lotus Game Board",
        options,
        Box::new(|_cc| Ok(Box::<LotusApp>::default())),
    )
}