use eframe::{egui, App, NativeOptions};
use egui::{
    vec2, Color32, Pos2, Response, Rgba, Sense, Shape, Stroke, Ui, Vec2, Widget,
};
use std::f32::consts::TAU; // TAU is 2 * PI

/// Main application state
struct LotusApp {
    player_petal_index: usize,
    num_petals: usize,
}

impl Default for LotusApp {
    fn default() -> Self {
        Self {
            player_petal_index: 0,
            num_petals: 8,
        }
    }
}

impl App for LotusApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Lotus Flower Game Board");
            ui.label("Click the buttons to move the player token.");
            ui.add_space(10.0);

            // --- UI Controls to move the player ---
            ui.horizontal(|ui| {
                if ui.button("Move Counter-Clockwise").clicked() {
                    self.player_petal_index =
                        (self.player_petal_index + self.num_petals - 1) % self.num_petals;
                }
                if ui.button("Move Clockwise").clicked() {
                    self.player_petal_index = (self.player_petal_index + 1) % self.num_petals;
                }
            });
            ui.label(format!(
                "Player is on petal index: {}",
                self.player_petal_index
            ));
            ui.add_space(50.0);

            // Center the widget
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                // Add our custom lotus widget, passing in the player's target index
                ui.add(LotusWidget::new(
                    self.num_petals,
                    100.0,
                    self.player_petal_index,
                ));
            });

            // Repaint continuously to see animations
            ctx.request_repaint();
        });
    }
}

/// Our custom widget.
struct LotusWidget {
    num_petals: usize,
    radius: f32,
    player_index: usize,
}

impl LotusWidget {
    pub fn new(num_petals: usize, radius: f32, player_index: usize) -> Self {
        Self {
            num_petals,
            radius,
            player_index,
        }
    }

    /// Helper function to get the "resting position" on a petal.
    fn get_petal_resting_pos(&self, index: usize, center: Pos2) -> Pos2 {
        let angle = (index as f32 / self.num_petals as f32) * TAU;
        // Use (sin, -cos) to have index 0 at the top (12 o'clock)
        let offset_vec = vec2(angle.sin(), -angle.cos()) * self.radius * 0.75;
        center + offset_vec
    }

    /// Helper to create a petal shape for drawing or hit-testing
    fn create_petal_shape(
        &self,
        center: Pos2,
        angle: f32,
        scale: f32,
        fill: Color32,
        stroke: Stroke,
    ) -> Shape {
        let p0 = center;
        let p3 = center;

        // Use vec2 (offset) instead of pos2 (absolute)
        let petal_width = self.radius * 0.5 * scale;
        let petal_length = self.radius * 1.2 * scale;
        let cp1_base = vec2(-petal_width, -petal_length);
        let cp2_base = vec2(petal_width, -petal_length);

        // Rotate the control points by the petal's angle
        let p1 = center + rotate_vec(cp1_base, angle);
        let p2 = center + rotate_vec(cp2_base, angle);

        Shape::CubicBezier(egui::epaint::CubicBezierShape {
            points: [p0, p1, p2, p3],
            closed: true,
            fill,
            // FIX: Use .into() to convert Stroke to PathStroke
            stroke: stroke.into(),
        })
    }
}

/// Implementation of the `Widget` trait for our `LotusWidget`.
impl Widget for LotusWidget {
    fn ui(self, ui: &mut Ui) -> Response {
        // 1. Allocate space for the widget
        let desired_size = vec2(self.radius * 2.5, self.radius * 2.5);
        let (rect, mut response) = ui.allocate_exact_size(desired_size, Sense::hover());

        let center = rect.center();
        let painter = ui.painter();
        let ctx = ui.ctx();

        // 2. Iterate and draw each petal
        for i in 0..self.num_petals {
            let petal_id = response.id.with(i);
            let angle = (i as f32 / self.num_petals as f32) * TAU;

            // --- Interaction: ---
            // First, get the *base* shape (scale 1.0) for hit-testing
            let base_shape = self.create_petal_shape(
                center,
                angle,
                1.0,
                Color32::TRANSPARENT,
                Stroke::NONE,
            );
            let hover_rect = base_shape.visual_bounding_rect();

            // Now, interact with that area
            let petal_response = ui.interact(hover_rect, petal_id, Sense::click_and_drag());
            let is_hovered = petal_response.hovered();
            
            // FIX: Use .is_pointer_button_down_on() instead of .down()
            let is_clicked = petal_response.is_pointer_button_down_on();

            // --- Animation: ---
            let scale_anim = ctx.animate_value_with_time(
                petal_id.with("scale"),
                if is_hovered { 1.2 } else { 1.0 },
                0.1,
            );

            // Animate click flash: 1.0 = white, 0.0 = normal
            let click_flash = ctx.animate_value_with_time(
                petal_id.with("click"),
                if is_clicked { 1.0 } else { 0.0 },
                0.1,
            );

            // --- Color Logic (using Rgba for lerp) ---
            let base_color = Rgba::from(Color32::from_rgb(255, 105, 180)); // Pink
            let hover_color = Rgba::from(Color32::from_rgb(255, 182, 193)); // Light Pink
            let click_color = Rgba::from(Color32::WHITE);

            // Interpolate between base and hover color
            let hover_progress = (scale_anim - 1.0) / 0.2; // 0.0 to 1.0
            let color_rgba = egui::lerp(base_color..=hover_color, hover_progress);

            // Interpolate between that and the click color
            let color_rgba_with_click = egui::lerp(color_rgba..=click_color, click_flash);
            let final_color: Color32 = color_rgba_with_click.into();

            // --- Drawing: ---
            // Now create the *actual* shape with the animated scale and color
            let petal_shape = self.create_petal_shape(
                center,
                angle,
                scale_anim,
                final_color,
                // Use from_black_alpha instead of with_alpha
                Stroke::new(1.0, Color32::from_black_alpha(60)),
            );

            painter.add(petal_shape);
            response |= petal_response;
        }

        // --- 3. Draw the Animated Player Token ---
        let target_pos = self.get_petal_resting_pos(self.player_index, center);
        let player_anim_id = response.id.with("player_token_pos");

        // FIX: Use animate_value for non-f32 types like Pos2
        let animated_pos = ctx.animate_value(player_anim_id, target_pos, 0.3);

        // Draw the player token (animated_pos is now Pos2)
        painter.circle_filled(
            animated_pos,
            10.0,
            Color32::from_rgb(255, 220, 0), // Yellow
        );
        painter.circle_stroke(
            animated_pos,
            10.0,
            // FIX: Corrected typo Color3 -> Color32
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
        viewport: egui::ViewportBuilder::default().with_inner_size(vec2(500.0, 500.0)),
        ..Default::default()
    };

    eframe::run_native(
        "Lotus Game Board",
        options,
        Box::new(|_cc| Ok(Box::<LotusApp>::default())),
    )
}