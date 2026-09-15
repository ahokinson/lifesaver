use crate::life::Life;
use macroquad::prelude::{screen_height, screen_width};

const TARGET_CELL_SIZE: f32 = 12.0;
const MIN_COLUMNS: usize = 48;
const MIN_ROWS: usize = 32;
const MAX_COLUMNS: usize = 320;
const MAX_ROWS: usize = 200;
const RESIZE_HYSTERESIS_CELLS: usize = 4;

/// Determines a dense but legible board from the active physical viewport.
///
/// The cap keeps work predictable on extremely high-resolution displays while
/// the aspect ratio always follows the screen instead of a fixed 16:10 board.
#[must_use]
pub fn grid_dimensions() -> (usize, usize) {
    (
        cells_for_extent(screen_width(), MIN_COLUMNS, MAX_COLUMNS),
        cells_for_extent(screen_height(), MIN_ROWS, MAX_ROWS),
    )
}

/// True when a resize materially changes the appropriate board dimensions.
#[must_use]
pub fn board_needs_resize(life: &Life) -> bool {
    let (columns, rows) = grid_dimensions();
    life.width().abs_diff(columns) >= RESIZE_HYSTERESIS_CELLS
        || life.height().abs_diff(rows) >= RESIZE_HYSTERESIS_CELLS
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn cells_for_extent(extent: f32, minimum: usize, maximum: usize) -> usize {
    // Screen dimensions are finite, positive values supplied by Macroquad.
    ((extent / TARGET_CELL_SIZE).round() as usize).clamp(minimum, maximum)
}

#[cfg(test)]
mod tests {
    use super::cells_for_extent;

    #[test]
    fn dimensions_follow_the_viewport_with_sane_bounds() {
        assert_eq!(cells_for_extent(1_920.0, 48, 320), 160);
        assert_eq!(cells_for_extent(1_080.0, 32, 200), 90);
        assert_eq!(cells_for_extent(1.0, 48, 320), 48);
        assert_eq!(cells_for_extent(9_999.0, 32, 200), 200);
    }
}
