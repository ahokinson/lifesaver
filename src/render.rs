use crate::life::Life;
use macroquad::prelude::*;

use crate::shapes::ShapeKind;

// Catppuccin Mocha
const BASE: Color = Color::new(0.118, 0.118, 0.180, 1.0); // #1e1e2e
const CELL_COLORS: [Color; 6] = [
    Color::new(0.537, 0.706, 0.980, 1.0), // blue #89b4fa
    Color::new(0.455, 0.780, 0.925, 1.0), // sapphire #74c7ec
    Color::new(0.537, 0.878, 0.749, 1.0), // green #a6e3a1
    Color::new(0.584, 0.886, 0.835, 1.0), // teal #94e2d5
    Color::new(0.796, 0.651, 0.969, 1.0), // mauve #cba6f7
    Color::new(0.961, 0.545, 0.659, 1.0), // pink #f38ba8
];

/// Draws the whole viewport without fixed text, grids, or panels.
pub fn draw(life: &Life, shapes: &[Option<ShapeKind>]) {
    clear_background(BASE);

    let cell_width = screen_width() / usize_to_f32(life.width());
    let cell_height = screen_height() / usize_to_f32(life.height());
    let inset = (cell_width.min(cell_height) * 0.14).clamp(0.7, 3.0);
    for y in 0..life.height() {
        for x in 0..life.width() {
            if !life.is_alive_at(x, y) {
                continue;
            }
            let index = y * life.width() + x;
            let color = cell_color(
                shapes[index],
                usize::from(life.age(x, y)),
                x,
                y,
                life.generation(),
            );
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

fn cell_color(shape: Option<ShapeKind>, age: usize, x: usize, y: usize, generation: u64) -> Color {
    let base = match shape {
        Some(ShapeKind::StillLife) => Color::new(0.976, 0.886, 0.686, 1.0), // yellow #f9e2af
        Some(ShapeKind::Oscillator) => Color::new(0.651, 0.890, 0.631, 1.0), // green #a6e3a1
        Some(ShapeKind::Glider) => Color::new(0.537, 0.706, 0.980, 1.0),    // blue #89b4fa
        Some(ShapeKind::Spaceship) => Color::new(0.796, 0.651, 0.969, 1.0), // mauve #cba6f7
        None => {
            let phase = usize::from(u8::try_from((generation / 7) % 6).unwrap_or(0));
            CELL_COLORS[(age / 7 + x / 13 + y / 17 + phase) % CELL_COLORS.len()]
        }
    };

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

fn usize_to_f32(value: usize) -> f32 {
    f32::from(u16::try_from(value).expect("LifeSaver grid dimensions fit in u16"))
}
