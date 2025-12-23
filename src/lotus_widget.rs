use eframe::egui::{
    self, Align2, Color32, FontId, Mesh, Pos2, Response, Rgba, Sense, Shape, Stroke, Ui, Vec2,
    Widget, vec2,
};
use std::f32::consts::TAU;

// --- Cached Geometry ---
#[derive(Clone)]
struct PetalInfo {
    base_shape: egui::epaint::CubicBezierShape,
    bounding_rect: egui::Rect,
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

/// Debug information for displaying in UI
#[derive(Clone, Default)]
pub struct LotusDebugInfo {
    pub pointer_pos: Option<Pos2>,
    pub widget_hovered: bool,
    pub hovered_petals: Vec<usize>,
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
        if petal == 0 {
            return "🎉".to_string();
        }
        if petal == 4 || petal == 8 {
            return "⚖️".to_string();
        }
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
        let widget_id = egui::Id::new("lotus_widget");

        // Properly allocate the entire available space for interaction
        let available = ui.available_size();
        let (rect, response) = ui.allocate_exact_size(
            available,
            Sense::hover().union(Sense::click())
        );

        let center = rect.center();
        let base_radius = rect.width().min(rect.height()) * 0.45;

        // --- Geometry Caching ---
        let mut cached_geo =
            ui.memory_mut(|mem| mem.data.get_persisted::<CachedGeometry>(widget_id).clone());

        if cached_geo.as_ref().map_or(true, |c| c.rect != rect) {
            let mut petals = Vec::new();
            for tier in (0..self.num_tiers).rev() {
                for petal in 0..self.num_petals_per_tier {
                    let total_index = tier * self.num_petals_per_tier + petal;
                    let tier_radius_factor = (tier as f32 + 1.0) / self.num_tiers as f32;
                    let tier_radius = base_radius * tier_radius_factor;
                    let tier_rotation =
                        (tier as f32 * (TAU / self.num_petals_per_tier as f32)) / 2.0;
                    let angle =
                        (petal as f32 / self.num_petals_per_tier as f32) * TAU + tier_rotation;

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

                    // Calculate bounding rect by sampling the bezier curve
                    let mut min_x = f32::INFINITY;
                    let mut min_y = f32::INFINITY;
                    let mut max_x = f32::NEG_INFINITY;
                    let mut max_y = f32::NEG_INFINITY;

                    for i in 0..=20 {
                        let t = i as f32 / 20.0;
                        let point = base_shape.sample(t);
                        min_x = min_x.min(point.x);
                        min_y = min_y.min(point.y);
                        max_x = max_x.max(point.x);
                        max_y = max_y.max(point.y);
                    }

                    let bounding_rect = egui::Rect::from_min_max(
                        Pos2::new(min_x, min_y),
                        Pos2::new(max_x, max_y),
                    );

                    petals.push(PetalInfo {
                        base_shape,
                        bounding_rect,
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

        // Collect all petal render data in original order
        let mut all_petals = Vec::new();

        // Get pointer position - use latest_pos() which always returns the last known position
        let pointer_pos = ui.input(|i| i.pointer.latest_pos());

        // Find the topmost hovered petal (check in reverse rendering order)
        let topmost_hovered_petal = if let Some(pos) = pointer_pos {
            // Check petals in reverse order (last rendered = topmost)
            cached_geo.petals.iter().rev().find(|petal_info| {
                petal_info.bounding_rect.contains(pos)
            }).map(|p| p.total_index)
        } else {
            None
        };

        // Collect debug info for UI display
        let mut hovered_petals = Vec::new();
        if let Some(idx) = topmost_hovered_petal {
            hovered_petals.push(idx);
        }

        for petal_info in &cached_geo.petals {
            let petal_id = response.id.with(petal_info.total_index);

            // Only the topmost petal is considered hovered
            let is_hovered = topmost_hovered_petal == Some(petal_info.total_index);

            // Track hover state transitions to trigger animation
            let hover_state_id = petal_id.with("hover_state");
            let was_hovered = ui.memory(|mem| mem.data.get_temp::<bool>(hover_state_id).unwrap_or(false));

            if is_hovered && !was_hovered {
                // Hover started - trigger animation by storing start time
                let time = ui.input(|i| i.time);
                ui.memory_mut(|mem| mem.data.insert_temp(petal_id.with("anim_start"), time));
            }
            ui.memory_mut(|mem| mem.data.insert_temp(hover_state_id, is_hovered));

            // Get animation progress (0.0 to 1.0 over animation duration)
            let anim_duration = 0.6; // Total animation duration in seconds
            let anim_start_time = ui.memory(|mem| mem.data.get_temp::<f64>(petal_id.with("anim_start")));
            let current_time = ui.input(|i| i.time);

            let (anim_progress, is_animating) = if let Some(start_time) = anim_start_time {
                let elapsed = current_time - start_time;
                if elapsed < anim_duration {
                    ctx.request_repaint(); // Keep animating
                    ((elapsed / anim_duration) as f32, true)
                } else {
                    (1.0, false)
                }
            } else {
                (0.0, false)
            };

            // Bounce animation: ease out elastic
            let scale = if is_animating && anim_progress < 1.0 {
                // Bounce curve: overshoot then settle
                let t = anim_progress;
                let bounce = if t < 0.5 {
                    // Scale up with overshoot
                    1.0 + (t * 2.0).powi(2) * 0.3
                } else {
                    // Scale down with dampening bounce
                    let t2 = (t - 0.5) * 2.0;
                    1.3 - t2 * 0.3 - (t2 * std::f32::consts::PI * 2.0).sin() * 0.05 * (1.0 - t2)
                };
                bounce
            } else {
                1.0
            };

            // Iridescent color flush
            let base_color_rgba = tier_colors
                .get(petal_info.tier)
                .cloned()
                .unwrap_or(Rgba::from(Color32::GRAY));

            let final_color = if is_animating && anim_progress < 1.0 {
                // Create iridescent effect by cycling through rainbow colors
                let hue_shift = anim_progress * 2.0; // Cycle through twice
                let saturation_boost = (1.0 - (anim_progress - 0.5).abs() * 2.0) * 0.6; // Peak at middle

                // Rainbow color based on animation progress
                let hue = (hue_shift * 360.0) % 360.0;
                let rainbow_color = hsv_to_rgb(hue, 0.7 + saturation_boost, 1.0);

                // Blend base color with rainbow, stronger in the middle of animation
                let blend_amount = (1.0 - (anim_progress - 0.5).abs() * 2.0).max(0.0);
                egui::lerp(base_color_rgba..=rainbow_color, blend_amount).into()
            } else {
                base_color_rgba.into()
            };

            let render_data = (petal_info.clone(), scale, final_color, is_animating);
            all_petals.push(render_data);
        }

        // Render all petals in original order (maintains z-order)
        for (petal_info, scale, final_color, is_animating) in all_petals {
            // Use slightly thicker stroke for animating petals
            let stroke = if is_animating {
                Stroke::new(2.0, Color32::from_black_alpha(100))
            } else {
                Stroke::new(1.0, Color32::from_black_alpha(60))
            };

            let (petal_mesh, petal_stroke_shape) = create_petal_mesh_from_base(
                &petal_info.base_shape,
                scale,
                final_color,
                stroke,
            );

            painter.add(petal_mesh);
            painter.add(petal_stroke_shape);
            painter.text(
                petal_info.text_pos,
                Align2::CENTER_CENTER,
                &petal_info.text,
                text_font.clone(),
                Color32::BLACK,
            );
        }

        // --- Player Token ---
        let player_petal_info = cached_geo
            .petals
            .iter()
            .find(|p| p.total_index == self.player_total_index)
            .unwrap();
        let target_pos = player_petal_info.text_pos;
        let player_anim_id = response.id.with("player_token_pos");
        let animated_x = ctx.animate_value_with_time(player_anim_id.with("x"), target_pos.x, 0.3);
        let animated_y = ctx.animate_value_with_time(player_anim_id.with("y"), target_pos.y, 0.3);
        let animated_pos = Pos2::new(animated_x, animated_y);

        let token_radius = (base_radius * 0.05).max(6.0);
        let token_stroke = (token_radius * 0.2).max(1.5);

        let time = ui.input(|i| i.time);
        let glow_anim_id = response.id.with("glow");
        let pulse =
            (ctx.animate_value_with_time(glow_anim_id, time as f32, 1.0) * 2.0).sin() * 0.5 + 0.5;
        let glow_radius = token_radius * (1.5 + pulse * 0.5);
        let glow_color = Color32::from_rgba_premultiplied(255, 220, 0, (pulse * 80.0) as u8);

        painter.circle_filled(animated_pos, glow_radius, glow_color);
        painter.circle_filled(animated_pos, token_radius, Color32::from_rgb(255, 220, 0));
        painter.circle_stroke(
            animated_pos,
            token_radius,
            Stroke::new(token_stroke, Color32::from_black_alpha(150)),
        );

        // Store debug info for UI display
        let debug_info = LotusDebugInfo {
            pointer_pos,
            widget_hovered: response.hovered(),
            hovered_petals,
        };
        ui.memory_mut(|mem| mem.data.insert_temp(widget_id.with("debug_info"), debug_info));

        response
    }
}

fn create_petal_mesh_from_base(
    base_shape: &egui::epaint::CubicBezierShape,
    scale: f32,
    fill_color: Color32,
    stroke: Stroke,
) -> (Mesh, Shape) {
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

/// Convert HSV color to RGB (Rgba)
/// h: 0-360, s: 0-1, v: 0-1
fn hsv_to_rgb(h: f32, s: f32, v: f32) -> Rgba {
    let c = v * s;
    let h_prime = h / 60.0;
    let x = c * (1.0 - ((h_prime % 2.0) - 1.0).abs());
    let m = v - c;

    let (r, g, b) = if h_prime < 1.0 {
        (c, x, 0.0)
    } else if h_prime < 2.0 {
        (x, c, 0.0)
    } else if h_prime < 3.0 {
        (0.0, c, x)
    } else if h_prime < 4.0 {
        (0.0, x, c)
    } else if h_prime < 5.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    Rgba::from_rgb(r + m, g + m, b + m)
}
