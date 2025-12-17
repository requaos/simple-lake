use eframe::egui::{self, vec2, Align2, Color32, FontId, Pos2, Response, Rgba, Sense, Shape, Stroke, Ui, Vec2, Widget, Mesh};
use std::f32::consts::TAU;

// --- Cached Geometry ---
#[derive(Clone)]
struct PetalInfo {
    base_shape: egui::epaint::CubicBezierShape,
    text_pos: Pos2,
    text: String,
    tier: usize,
    petal: usize,
    total_index: usize,
}

#[derive(Clone)]
struct CachedGeometry {
    petals: Vec<PetalInfo>,
    rect: egui::Rect,
}

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

    /// Helper function to get the text for a specific petal
    fn get_petal_text(tier: usize, petal: usize) -> String {
        if petal == 0 { return "🎉".to_string(); }
        if petal == 4 || petal == 8 { return "⚖️".to_string(); }
        match tier {
            0 => "💀".to_string(),
            1 => "⚠️".to_string(),
            2 => "💼".to_string(),
            3 => "🍲".to_string(),
            4 => "🏆".to_string(),
            _ => "??".to_string(),
        }
    }
}

/// Implementation of the `Widget` trait for our `LotusWidget`.
impl Widget for LotusWidget {
    fn ui(self, ui: &mut Ui) -> Response {
        let widget_id = ui.id().with("lotus_widget");
        let mut response = ui.allocate_rect(ui.available_rect_before_wrap(), Sense::hover());
        let rect = response.rect;
        let center = rect.center();
        let base_radius = rect.width().min(rect.height()) * 0.45;

        // --- Geometry Caching ---
        let mut cached_geo = ui.memory_mut(|mem| mem.data.get_persisted::<CachedGeometry>(widget_id).clone());

        if cached_geo.as_ref().map_or(true, |c| c.rect != rect) {
            let mut petals = Vec::new();
            for tier in (0..self.num_tiers).rev() {
                for petal in 0..self.num_petals_per_tier {
                    let total_index = tier * self.num_petals_per_tier + petal;
                    let tier_radius_factor = (tier as f32 + 1.0) / self.num_tiers as f32;
                    let tier_radius = base_radius * tier_radius_factor;
                    let tier_rotation = (tier as f32 * (TAU / self.num_petals_per_tier as f32)) / 2.0;
                    let angle = (petal as f32 / self.num_petals_per_tier as f32) * TAU + tier_rotation;

                    let p0 = center;
                    let p3 = center;
                    let petal_width = tier_radius * 0.9;
                    let petal_length = tier_radius * 1.1;
                    let cp1_base = vec2(-petal_width, -petal_length);
                    let cp2_base = vec2(petal_width, -petal_length);
                    let p1 = center + rotate_vec(cp1_base, angle);
                    let p2 = center + rotate_vec(cp2_base, angle);

                    let base_shape = egui::epaint::CubicBezierShape {
                        points: [p0, p1, p2, p3],
                        closed: true,
                        fill: Color32::TRANSPARENT,
                        stroke: Stroke::NONE.into(),
                    };

                    let offset_vec = vec2(angle.sin(), -angle.cos()) * tier_radius * 0.75;
                    let text_pos = center + offset_vec;

                    petals.push(PetalInfo {
                        base_shape,
                        text_pos,
                        text: Self::get_petal_text(tier, petal),
                        tier,
                        petal,
                        total_index,
                    });
                }
            }
            let new_cache = CachedGeometry { petals, rect };
            ui.memory_mut(|mem| mem.data.insert_persisted(widget_id, new_cache.clone()));
            cached_geo = Some(new_cache);
        }

        let cached_geo = cached_geo.unwrap();
        let painter = ui.painter();
        let ctx = ui.ctx();

        let tier_colors = [
            Rgba::from(Color32::from_rgb(80, 80, 80)),
            Rgba::from(Color32::from_rgb(255, 100, 100)),
            Rgba::from(Color32::from_rgb(255, 180, 105)),
            Rgba::from(Color32::from_rgb(105, 200, 255)),
            Rgba::from(Color32::from_rgb(255, 220, 100)),
        ];
        let text_font = FontId::proportional(16.0);

        for petal_info in &cached_geo.petals {
            let petal_id = response.id.with(petal_info.total_index);
            let hover_rect = petal_info.base_shape.visual_bounding_rect();
            let petal_response = ui.interact(hover_rect, petal_id, Sense::hover());
            let is_hovered = petal_response.hovered();

            let scale_anim = ctx.animate_value_with_time(petal_id.with("scale"), if is_hovered { 1.2 } else { 1.0 }, 0.1);

            let base_color_rgba = tier_colors.get(petal_info.tier).cloned().unwrap_or(Rgba::from(Color32::GRAY));
            let hover_color_rgba = egui::lerp(base_color_rgba..=Rgba::WHITE, 0.4);
            let hover_progress = (scale_anim - 1.0) / 0.2;
            let color_rgba = egui::lerp(base_color_rgba..=hover_color_rgba, hover_progress);
            let final_color: Color32 = color_rgba.into();

            let (petal_mesh, petal_stroke_shape) = create_petal_mesh_from_base(&petal_info.base_shape, scale_anim, final_color, Stroke::new(1.0, Color32::from_black_alpha(60)));

            painter.add(petal_mesh);
            painter.add(petal_stroke_shape);
            painter.text(petal_info.text_pos, Align2::CENTER_CENTER, &petal_info.text, text_font.clone(), Color32::BLACK);

            response |= petal_response;
        }

        // --- Player Token ---
        let player_petal_info = cached_geo.petals.iter().find(|p| p.total_index == self.player_total_index).unwrap();
        let target_pos = player_petal_info.text_pos;
        let player_anim_id = response.id.with("player_token_pos");
        let animated_x = ctx.animate_value_with_time(player_anim_id.with("x"), target_pos.x, 0.3);
        let animated_y = ctx.animate_value_with_time(player_anim_id.with("y"), target_pos.y, 0.3);
        let animated_pos = Pos2::new(animated_x, animated_y);

        let token_radius = (base_radius * 0.05).max(6.0);
        let token_stroke = (token_radius * 0.2).max(1.5);

        let time = ui.input(|i| i.time);
        let glow_anim_id = response.id.with("glow");
        let pulse = (ctx.animate_value_with_time(glow_anim_id, time as f32, 1.0) * 2.0).sin() * 0.5 + 0.5;
        let glow_radius = token_radius * (1.5 + pulse * 0.5);
        let glow_color = Color32::from_rgba_premultiplied(255, 220, 0, (pulse * 80.0) as u8);

        painter.circle_filled(animated_pos, glow_radius, glow_color);
        painter.circle_filled(animated_pos, token_radius, Color32::from_rgb(255, 220, 0));
        painter.circle_stroke(animated_pos, token_radius, Stroke::new(token_stroke, Color32::from_black_alpha(150)));

        response
    }
}

fn create_petal_mesh_from_base(base_shape: &egui::epaint::CubicBezierShape, scale: f32, fill_color: Color32, stroke: Stroke) -> (Mesh, Shape) {
    let mut scaled_points = base_shape.points;
    let center = scaled_points[0];
    for i in 1..4 {
        scaled_points[i] = center + (scaled_points[i] - center) * scale;
    }

    let bezier = egui::epaint::CubicBezierShape {
        points: scaled_points,
        closed: true,
        fill: Color32::TRANSPARENT,
        stroke: stroke.into(),
    };

    let mut mesh = Mesh::default();
    let center_color = fill_color;
    let edge_color = Color32::from_rgba_premultiplied(
        (fill_color.r() as f32 * 0.4) as u8,
        (fill_color.g() as f32 * 0.4) as u8,
        (fill_color.b() as f32 * 0.4) as u8,
        (fill_color.a() as f32 * 0.8) as u8,
    );

    const N_POINTS: usize = 20;
    mesh.colored_vertex(center, center_color);

    for i in 0..=N_POINTS {
        let t = i as f32 / N_POINTS as f32;
        let pos = bezier.sample(t);
        mesh.colored_vertex(pos, edge_color);
    }

    for i in 1..=N_POINTS {
        mesh.add_triangle(0, i as u32, (i + 1) as u32);
    }
    mesh.add_triangle(0, N_POINTS as u32, 1);

    (mesh, Shape::CubicBezier(bezier))
}

/// Helper function to rotate a Vec2
fn rotate_vec(v: Vec2, angle: f32) -> Vec2 {
    let (sin, cos) = angle.sin_cos();
    vec2(v.x * cos - v.y * sin, v.x * sin + v.y * cos)
}