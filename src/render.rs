use crate::life::Life;
use macroquad::prelude::*;

use crate::shapes::ShapeKind;

// Catppuccin Mocha
const BASE: Color = Color::new(0.118, 0.118, 0.180, 1.0); // #1e1e2e
const OVERLAY2: Color = Color::new(0.576, 0.600, 0.698, 1.0); // #9399b2
const GREEN: Color = Color::new(0.651, 0.890, 0.631, 1.0); // #a6e3a1
const BLUE: Color = Color::new(0.537, 0.706, 0.980, 1.0); // #89b4fa
const MAUVE: Color = Color::new(0.796, 0.651, 0.969, 1.0); // #cba6f7

const UNRECOGNIZED_COLOR_TRANSITION_GENERATIONS: u64 = 60;
const UNRECOGNIZED_COLORS: [Color; 11] = [
    Color::new(0.961, 0.878, 0.863, 1.0), // rosewater #f5e0dc
    Color::new(0.949, 0.804, 0.804, 1.0), // flamingo #f2cdcd
    Color::new(0.961, 0.761, 0.906, 1.0), // pink #f5c2e7
    Color::new(0.953, 0.545, 0.659, 1.0), // red #f38ba8
    Color::new(0.922, 0.627, 0.675, 1.0), // maroon #eba0ac
    Color::new(0.980, 0.702, 0.529, 1.0), // peach #fab387
    Color::new(0.976, 0.886, 0.686, 1.0), // yellow #f9e2af
    Color::new(0.580, 0.886, 0.835, 1.0), // teal #94e2d5
    Color::new(0.537, 0.863, 0.922, 1.0), // sky #89dceb
    Color::new(0.455, 0.780, 0.925, 1.0), // sapphire #74c7ec
    Color::new(0.706, 0.745, 0.996, 1.0), // lavender #b4befe
];

/// Draws the whole viewport without fixed text, grids, or panels.
pub fn draw(life: &Life, shapes: &[Option<ShapeKind>]) {
    clear_background(BASE);

    let cell_width = screen_width() / usize_to_f32(life.width());
    let cell_height = screen_height() / usize_to_f32(life.height());
    let inset = (cell_width.min(cell_height) * 0.14).clamp(0.7, 3.0);
    let generation = life.generation();
    let unrecognized_color = unrecognized_color(generation);
    for y in 0..life.height() {
        for x in 0..life.width() {
            if !life.is_alive_at(x, y) {
                continue;
            }
            let index = y * life.width() + x;
            let color = cell_color(shapes[index], x, y, generation, unrecognized_color);
            let position_x = usize_to_f32(x).mul_add(cell_width, 0.0);
            let position_y = usize_to_f32(y).mul_add(cell_height, 0.0);
            let glow = Color::new(color.r, color.g, color.b, 0.16);
            draw_rectangle(
                position_x - inset,
                position_y - inset,
                cell_width + inset * 2.0,
                cell_height + inset * 2.0,
                glow,
            );
            draw_rectangle(
                position_x + inset,
                position_y + inset,
                (cell_width - inset * 2.0).max(1.0),
                (cell_height - inset * 2.0).max(1.0),
                color,
            );
        }
    }
}

fn cell_color(
    shape: Option<ShapeKind>,
    x: usize,
    y: usize,
    generation: u64,
    unrecognized_color: Color,
) -> Color {
    let base = shape_color(shape, unrecognized_color);

    // Stable structures still change luminosity in time and never leave a
    // permanently bright cell while preserving their semantic color.
    let generation_phase = f32::from(u16::try_from(generation % 4096).unwrap_or(0));
    let phase =
        usize_to_f32(y).mul_add(0.09, usize_to_f32(x).mul_add(0.12, generation_phase * 0.15));
    let brightness = 0.79 + 0.21 * (phase.sin() + 1.0) / 2.0;
    Color::new(
        base.r * brightness,
        base.g * brightness,
        base.b * brightness,
        1.0,
    )
}

const fn shape_color(shape: Option<ShapeKind>, unrecognized_color: Color) -> Color {
    match shape {
        Some(ShapeKind::StillLife) => OVERLAY2,
        Some(ShapeKind::Oscillator) => GREEN,
        Some(ShapeKind::Glider) => BLUE,
        Some(ShapeKind::Spaceship) => MAUVE,
        Some(ShapeKind::Unrecognized) | None => unrecognized_color,
    }
}

/// Returns the shared fallback hue, blending between unused Mocha accents.
fn unrecognized_color(generation: u64) -> Color {
    let color_count = u64::try_from(UNRECOGNIZED_COLORS.len()).expect("palette length fits in u64");
    let from_index =
        usize::try_from((generation / UNRECOGNIZED_COLOR_TRANSITION_GENERATIONS) % color_count)
            .expect("palette index fits in usize");
    let to_index = (from_index + 1) % UNRECOGNIZED_COLORS.len();
    let progress = f32::from(
        u8::try_from(generation % UNRECOGNIZED_COLOR_TRANSITION_GENERATIONS)
            .expect("transition duration fits in u8"),
    ) / f32::from(
        u8::try_from(UNRECOGNIZED_COLOR_TRANSITION_GENERATIONS)
            .expect("transition duration fits in u8"),
    );

    lerp_color(
        UNRECOGNIZED_COLORS[from_index],
        UNRECOGNIZED_COLORS[to_index],
        progress,
    )
}

fn lerp_color(from: Color, to: Color, progress: f32) -> Color {
    Color::new(
        (to.r - from.r).mul_add(progress, from.r),
        (to.g - from.g).mul_add(progress, from.g),
        (to.b - from.b).mul_add(progress, from.b),
        (to.a - from.a).mul_add(progress, from.a),
    )
}

fn usize_to_f32(value: usize) -> f32 {
    f32::from(u16::try_from(value).expect("LifeSaver grid dimensions fit in u16"))
}

#[cfg(test)]
mod tests {
    use super::{
        OVERLAY2, UNRECOGNIZED_COLOR_TRANSITION_GENERATIONS, UNRECOGNIZED_COLORS, shape_color,
        unrecognized_color,
    };
    use crate::shapes::ShapeKind;
    use macroquad::prelude::Color;

    #[test]
    fn maps_still_lifes_to_overlay2_and_unrecognized_shapes_to_the_current_accent() {
        assert_eq!(
            shape_color(Some(ShapeKind::StillLife), UNRECOGNIZED_COLORS[0]),
            OVERLAY2
        );
        assert_eq!(
            shape_color(Some(ShapeKind::Unrecognized), UNRECOGNIZED_COLORS[0]),
            UNRECOGNIZED_COLORS[0]
        );
    }

    #[test]
    fn unrecognized_shapes_blend_through_every_available_accent() {
        assert_eq!(unrecognized_color(0), UNRECOGNIZED_COLORS[0]);
        assert_eq!(
            unrecognized_color(UNRECOGNIZED_COLOR_TRANSITION_GENERATIONS),
            UNRECOGNIZED_COLORS[1]
        );
        assert_eq!(
            unrecognized_color(
                UNRECOGNIZED_COLOR_TRANSITION_GENERATIONS
                    * u64::try_from(UNRECOGNIZED_COLORS.len() - 1).expect("palette length fits"),
            ),
            UNRECOGNIZED_COLORS[UNRECOGNIZED_COLORS.len() - 1]
        );
        assert_eq!(
            unrecognized_color(
                UNRECOGNIZED_COLOR_TRANSITION_GENERATIONS
                    * u64::try_from(UNRECOGNIZED_COLORS.len()).expect("palette length fits"),
            ),
            UNRECOGNIZED_COLORS[0]
        );

        let halfway = unrecognized_color(UNRECOGNIZED_COLOR_TRANSITION_GENERATIONS / 2);
        let expected = Color::new(0.955, 0.841, 0.8335, 1.0);
        assert!((halfway.r - expected.r).abs() < 0.0001);
        assert!((halfway.g - expected.g).abs() < 0.0001);
        assert!((halfway.b - expected.b).abs() < 0.0001);
    }
}
