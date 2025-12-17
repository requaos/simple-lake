use eframe::egui::{self, vec2, Align2, Color32, FontId, Pos2, Response, Rgba, Sense, Shape, Stroke, Ui, Vec2, Widget, Mesh};
use eframe::epaint::PathStroke;
use std::f32::consts::TAU;

/// Our custom widget.
/// This widget is "dumb" - it just receives a total_index and renders it.
pub struct LotusWidget {
    num_tiers: usize,
    num_petals_per_tier: usize,
    player_total_index: usize,
}

impl LotusWidget {
    pub fn new(num_tiers: usize, num_petals_per_tier: usize, player_total_index: usize) -> Self {
        Self {
            num_tiers,
            num_petals_per_tier,
            player_total_index,
        }
    }

    /// Helper function to get the "resting position" on a petal.
    fn get_petal_resting_pos(
        &self,
        total_index: usize,
        center: Pos2,
        base_radius: f32,
    ) -> Pos2 {
        let tier = total_index / self.num_petals_per_tier;
        let petal = total_index % self.num_petals_per_tier;

        // Calculate this tier's radius (from 1/N to N/N)
        let tier_radius_factor = (tier as f32 + 1.0) / self.num_tiers as f32;
        let tier_radius = base_radius * tier_radius_factor;

        // Offset each tier's rotation by half a petal
        let tier_rotation = (tier as f32 * (TAU / self.num_petals_per_tier as f32)) / 2.0;
        let angle = (petal as f32 / self.num_petals_per_tier as f32) * TAU + tier_rotation;

        // Use (sin, -cos) to have index 0 at the top (12 o'clock)
        // Place it 75% of the way out on that tier's radius
        let offset_vec = vec2(angle.sin(), -angle.cos()) * tier_radius * 0.75;
        center + offset_vec
    }

    /// Helper to create a petal shape for drawing or hit-testing
    fn create_petal_mesh(
        &self,
        center: Pos2,
        radius: f32, // This is now the radius for the *current tier*
        angle: f32,
        scale: f32,
        fill_color: Color32,
        stroke: Stroke,
    ) -> (Mesh, Shape) {
        let p0 = center;
        let p3 = center;

        // Petal size is relative to the tier's radius
        let petal_width = radius * 0.9 * scale;
        let petal_length = radius * 1.1 * scale;

        let cp1_base = vec2(-petal_width, -petal_length);
        let cp2_base = vec2(petal_width, -petal_length);

        // Rotate the control points by the angle
        let p1 = center + rotate_vec(cp1_base, angle);
        let p2 = center + rotate_vec(cp2_base, angle);

        let bezier = egui::epaint::CubicBezierShape {
            points: [p0, p1, p2, p3],
            closed: true,
            fill: Color32::TRANSPARENT, // We use the mesh for fill
            stroke: stroke.into(),
        };

        // --- Create Gradient Mesh ---
        let mut mesh = Mesh::default();
        let center_color = fill_color;
        let edge_color = Color32::from_rgba_premultiplied(
            (fill_color.r() as f32 * 0.4) as u8,
            (fill_color.g() as f32 * 0.4) as u8,
            (fill_color.b() as f32 * 0.4) as u8,
            (fill_color.a() as f32 * 0.8) as u8,
        );

        // Approximate the bezier with a fan of triangles from the center
        const N_POINTS: usize = 20;
        mesh.colored_vertex(p0, center_color); // Center vertex

        for i in 0..=N_POINTS {
            let t = i as f32 / N_POINTS as f32;
            let pos = bezier.sample(t);
            mesh.colored_vertex(pos, edge_color);
        }

        // Create the triangles
        for i in 1..=N_POINTS {
            mesh.add_triangle(0, i as u32, (i + 1) as u32);
        }
        mesh.add_triangle(0, N_POINTS as u32, 1); // close the loop

        (mesh, Shape::CubicBezier(bezier))
    }

    /// Helper function to get the text for a specific petal
    fn get_petal_text(&self, tier: usize, petal: usize, _total_index: usize) -> String {
        if petal == 0 {
            return "🎉".to_string(); // New Year
        }
        if petal == 4 || petal == 8 {
            return "⚖️".to_string(); // SCS Review
        }

        match tier {
            0 => "💀".to_string(), // Tier D (Blacklisted)
            1 => "⚠️".to_string(), // Tier C (Warning)
            2 => "💼".to_string(), // Tier B (Standard Event)
            3 => "🍲".to_string(), // Tier A (Party Banquet)
            4 => "🏆".to_string(), // Tier A+ (Honored)
            _ => "??".to_string(),
        }
    }
}

/// Implementation of the `Widget` trait for our `LotusWidget`.
impl Widget for LotusWidget {
    fn ui(self, ui: &mut Ui) -> Response {
        // 1. Allocate available space for the widget
        let mut response =
            ui.allocate_rect(ui.available_rect_before_wrap(), Sense::hover());
        let rect = response.rect; // Get the Rect *from* the Response

        // 2. Calculate dynamic radius based on the allocated space
        let base_radius = rect.width().min(rect.height()) * 0.45;
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

        let text_font = FontId::proportional(16.0); // Increased for icons

        // 3. Iterate and draw each petal for each tier
        for tier in (0..self.num_tiers).rev() {
            let tier_radius_factor = (tier as f32 + 1.0) / self.num_tiers as f32;
            let tier_radius = base_radius * tier_radius_factor;
            let tier_rotation = (tier as f32 * (TAU / self.num_petals_per_tier as f32)) / 2.0;

            let base_color_rgba = tier_colors.get(tier).cloned().unwrap_or(Rgba::from(Color32::GRAY));
            let hover_color_rgba = egui::lerp(base_color_rgba..=Rgba::WHITE, 0.4);

            for petal in 0..self.num_petals_per_tier {
                let petal_total_index = tier * self.num_petals_per_tier + petal;
                let petal_id = response.id.with(petal_total_index);
                let angle = (petal as f32 / self.num_petals_per_tier as f32) * TAU + tier_rotation;

                // --- Interaction & Animation ---
                let (_, boundary_shape) = self.create_petal_mesh(center, tier_radius, angle, 1.0, Color32::TRANSPARENT, Stroke::NONE);
                let hover_rect = boundary_shape.visual_bounding_rect();
                let petal_response = ui.interact(hover_rect, petal_id, Sense::hover());
                let is_hovered = petal_response.hovered();

                let scale_anim = ctx.animate_value_with_time(petal_id.with("scale"), if is_hovered { 1.2 } else { 1.0 }, 0.1);
                let hover_progress = (scale_anim - 1.0) / 0.2;
                let color_rgba = egui::lerp(base_color_rgba..=hover_color_rgba, hover_progress);
                let final_color: Color32 = color_rgba.into();

                // --- Drawing ---
                let (petal_mesh, petal_stroke_shape) = self.create_petal_mesh(
                    center,
                    tier_radius,
                    angle,
                    scale_anim,
                    final_color,
                    Stroke::new(1.0, Color32::from_black_alpha(60)),
                );

                painter.add(petal_mesh);
                painter.add(petal_stroke_shape);

                // --- DRAW PETAL TEXT (ICONS) ---
                let petal_text_pos = self.get_petal_resting_pos(petal_total_index, center, base_radius);
                let text = self.get_petal_text(tier, petal, petal_total_index);

                painter.text(
                    petal_text_pos,
                    Align2::CENTER_CENTER,
                    text,
                    text_font.clone(),
                    Color32::BLACK,
                );

                response |= petal_response;
            }
        }

        // --- 4. Draw the Animated Player Token with Glow ---
        let target_pos = self.get_petal_resting_pos(self.player_total_index, center, base_radius);
        let player_anim_id = response.id.with("player_token_pos");

        let animated_x = ctx.animate_value_with_time(player_anim_id.with("x"), target_pos.x, 0.3);
        let animated_y = ctx.animate_value_with_time(player_anim_id.with("y"), target_pos.y, 0.3);
        let animated_pos = Pos2::new(animated_x, animated_y);

        let token_radius = (base_radius * 0.05).max(6.0);
        let token_stroke = (token_radius * 0.2).max(1.5);

        // Pulsing glow effect
        let time = ui.input(|i| i.time);
        let glow_anim_id = response.id.with("glow");
        let pulse = (ctx.animate_value_with_time(glow_anim_id, time as f32, 1.0) * 2.0).sin() * 0.5 + 0.5; // Slow pulse
        let glow_radius = token_radius * (1.5 + pulse * 0.5);
        let glow_color = Color32::from_rgba_premultiplied(255, 220, 0, (pulse * 80.0) as u8);
        painter.circle_filled(animated_pos, glow_radius, glow_color);

        // Draw the main player token
        painter.circle_filled(animated_pos, token_radius, Color32::from_rgb(255, 220, 0));
        painter.circle_stroke(animated_pos, token_radius, Stroke::new(token_stroke, Color32::from_black_alpha(150)));

        response
    }
}

/// Helper function to rotate a Vec2
fn rotate_vec(v: Vec2, angle: f32) -> Vec2 {
    let (sin, cos) = angle.sin_cos();
    vec2(v.x * cos - v.y * sin, v.x * sin + v.y * cos)
}