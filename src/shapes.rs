use crate::life::Life;

const BLINKER: &[(isize, isize)] = &[(0, 0), (1, 0), (2, 0)];
const TOAD: &[(isize, isize)] = &[(1, 0), (2, 0), (3, 0), (0, 1), (1, 1), (2, 1)];
const BEACON: &[(isize, isize)] = &[
    (0, 0),
    (1, 0),
    (0, 1),
    (1, 1),
    (2, 2),
    (3, 2),
    (2, 3),
    (3, 3),
];
const GLIDER: &[(isize, isize)] = &[(1, 0), (2, 1), (0, 2), (1, 2), (2, 2)];
const LIGHTWEIGHT_SPACESHIP: &[(isize, isize)] = &[
    (1, 0),
    (4, 0),
    (0, 1),
    (0, 2),
    (4, 2),
    (0, 3),
    (1, 3),
    (2, 3),
    (3, 3),
];

/// Semantic classes used by the renderer's Catppuccin colors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShapeKind {
    StillLife,
    Oscillator,
    Glider,
    Spaceship,
    Unrecognized,
}

/// Labels every live cell with either a recognized Life structure or a
/// coherent fallback class for its connected component.
#[must_use]
pub fn classify(life: &Life) -> Vec<Option<ShapeKind>> {
    let mut labels = vec![None; life.width() * life.height()];
    let mut visited = vec![false; labels.len()];
    let width = life.width();
    let height = life.height();

    for y in 0..height {
        for x in 0..width {
            let start = y * width + x;
            if visited[start] || !life.is_alive_at(x, y) {
                continue;
            }

            let component = collect_component(life, &mut visited, x, y);
            let shape = is_still_life(life, &component)
                .then_some(ShapeKind::StillLife)
                .or_else(|| classify_component(&component))
                .unwrap_or(ShapeKind::Unrecognized);
            for (index, _) in component {
                labels[index] = Some(shape);
            }
        }
    }
    label_isolated_pattern(
        life,
        &mut labels,
        ShapeKind::Spaceship,
        LIGHTWEIGHT_SPACESHIP,
    );
    labels
}

fn collect_component(
    life: &Life,
    visited: &mut [bool],
    start_x: usize,
    start_y: usize,
) -> Vec<(usize, (isize, isize))> {
    let width = life.width();
    let height = life.height();
    let mut component = Vec::new();
    let mut stack = vec![(
        start_x,
        start_y,
        usize_to_isize(start_x),
        usize_to_isize(start_y),
    )];
    visited[start_y * width + start_x] = true;

    while let Some((cell_x, cell_y, unwrapped_x, unwrapped_y)) = stack.pop() {
        component.push((cell_y * width + cell_x, (unwrapped_x, unwrapped_y)));
        for offset_y in -1..=1 {
            for offset_x in -1..=1 {
                if offset_x == 0 && offset_y == 0 {
                    continue;
                }
                let neighbour_x = wrap_coordinate(cell_x, offset_x, width);
                let neighbour_y = wrap_coordinate(cell_y, offset_y, height);
                let neighbour = neighbour_y * width + neighbour_x;
                if !visited[neighbour] && life.is_alive_at(neighbour_x, neighbour_y) {
                    visited[neighbour] = true;
                    stack.push((
                        neighbour_x,
                        neighbour_y,
                        unwrapped_x + offset_x,
                        unwrapped_y + offset_y,
                    ));
                }
            }
        }
    }
    component
}

fn label_isolated_pattern(
    life: &Life,
    labels: &mut [Option<ShapeKind>],
    kind: ShapeKind,
    pattern: &[(isize, isize)],
) {
    for transform in 0..8 {
        let pattern = normalise(
            pattern
                .iter()
                .map(|point| transform_point(*point, transform))
                .collect(),
        );
        let pattern = pattern
            .iter()
            .map(|(x, y)| {
                (
                    usize::try_from(*x).expect("normalised x coordinate is non-negative"),
                    usize::try_from(*y).expect("normalised y coordinate is non-negative"),
                )
            })
            .collect::<Vec<_>>();
        let max_x = pattern.iter().map(|(x, _)| *x).max().unwrap_or(0);
        let max_y = pattern.iter().map(|(_, y)| *y).max().unwrap_or(0);

        for anchor_y in 0..life.height() {
            for anchor_x in 0..life.width() {
                if !is_isolated_pattern(life, anchor_x, anchor_y, &pattern, max_x, max_y) {
                    continue;
                }
                for &(x, y) in &pattern {
                    let x = wrap_coordinate(anchor_x, usize_to_isize(x), life.width());
                    let y = wrap_coordinate(anchor_y, usize_to_isize(y), life.height());
                    labels[y * life.width() + x] = Some(kind);
                }
            }
        }
    }
}

fn is_isolated_pattern(
    life: &Life,
    anchor_x: usize,
    anchor_y: usize,
    pattern: &[(usize, usize)],
    max_x: usize,
    max_y: usize,
) -> bool {
    for y in 0..=max_y + 2 {
        for x in 0..=max_x + 2 {
            let expected = x
                .checked_sub(1)
                .zip(y.checked_sub(1))
                .is_some_and(|point| pattern.contains(&point));
            let x = wrap_coordinate(anchor_x, usize_to_isize(x) - 1, life.width());
            let y = wrap_coordinate(anchor_y, usize_to_isize(y) - 1, life.height());
            if life.is_alive_at(x, y) != expected {
                return false;
            }
        }
    }
    true
}

/// Returns whether this connected component survives unchanged for one Life
/// generation. Any birth must be adjacent to a live cell, so checking each
/// component cell's neighbourhood covers every cell that could change. The
/// simulation's own next-state rule is used to prevent detector drift.
fn is_still_life(life: &Life, component: &[(usize, (isize, isize))]) -> bool {
    for &(index, _) in component {
        let x = index % life.width();
        let y = index / life.width();
        if !life.will_be_alive_at(x, y) {
            return false;
        }

        for offset_y in -1..=1 {
            for offset_x in -1..=1 {
                if offset_x == 0 && offset_y == 0 {
                    continue;
                }
                let neighbour_x = wrap_coordinate(x, offset_x, life.width());
                let neighbour_y = wrap_coordinate(y, offset_y, life.height());
                if !life.is_alive_at(neighbour_x, neighbour_y)
                    && life.will_be_alive_at(neighbour_x, neighbour_y)
                {
                    return false;
                }
            }
        }
    }
    true
}

fn classify_component(component: &[(usize, (isize, isize))]) -> Option<ShapeKind> {
    let points = component
        .iter()
        .map(|(_, point)| *point)
        .collect::<Vec<_>>();
    [
        (ShapeKind::Oscillator, BLINKER),
        (ShapeKind::Oscillator, TOAD),
        (ShapeKind::Oscillator, BEACON),
        (ShapeKind::Glider, GLIDER),
        (ShapeKind::Spaceship, LIGHTWEIGHT_SPACESHIP),
    ]
    .into_iter()
    .find_map(|(kind, pattern)| is_shape(&points, pattern).then_some(kind))
}

fn is_shape(points: &[(isize, isize)], pattern: &[(isize, isize)]) -> bool {
    let expected = normalise(pattern.to_vec());
    (0..8).any(|transform| {
        normalise(
            points
                .iter()
                .map(|point| transform_point(*point, transform))
                .collect(),
        ) == expected
    })
}

const fn transform_point((x, y): (isize, isize), transform: u8) -> (isize, isize) {
    match transform {
        0 => (x, y),
        1 => (x, -y),
        2 => (-x, y),
        3 => (-x, -y),
        4 => (y, x),
        5 => (y, -x),
        6 => (-y, x),
        _ => (-y, -x),
    }
}

fn usize_to_isize(value: usize) -> isize {
    isize::try_from(value).expect("LifeSaver grid dimensions fit in isize")
}

fn wrap_coordinate(value: usize, offset: isize, dimension: usize) -> usize {
    let value = usize_to_isize(value);
    let dimension = usize_to_isize(dimension);
    usize::try_from((value + offset).rem_euclid(dimension))
        .expect("wrapped coordinate is non-negative")
}

fn normalise(mut points: Vec<(isize, isize)>) -> Vec<(isize, isize)> {
    let min_x = points.iter().map(|(x, _)| *x).min().unwrap_or(0);
    let min_y = points.iter().map(|(_, y)| *y).min().unwrap_or(0);
    for (x, y) in &mut points {
        *x -= min_x;
        *y -= min_y;
    }
    points.sort_unstable();
    points
}

#[cfg(test)]
mod tests {
    use super::{GLIDER, LIGHTWEIGHT_SPACESHIP, ShapeKind, classify};
    use crate::life::Life;

    fn board_with(points: &[(isize, isize)]) -> Life {
        let mut life = Life::new(20, 20);
        for &(x, y) in points {
            life.set_alive(
                usize::try_from(x).expect("test x coordinate is non-negative"),
                usize::try_from(y).expect("test y coordinate is non-negative"),
                true,
            );
        }
        life
    }

    #[test]
    fn classifies_a_still_life() {
        let life = board_with(&[(4, 3), (5, 3), (4, 4), (5, 4)]);
        let labels = classify(&life);
        assert_eq!(labels[3 * life.width() + 4], Some(ShapeKind::StillLife));
    }

    #[test]
    fn classifies_an_unlisted_still_life() {
        let cells = [(5, 4), (4, 5), (6, 5), (5, 6)]; // tub
        let life = board_with(&cells);
        let labels = classify(&life);

        for (x, y) in cells {
            let x = usize::try_from(x).expect("test x coordinate is non-negative");
            let y = usize::try_from(y).expect("test y coordinate is non-negative");
            assert_eq!(labels[y * life.width() + x], Some(ShapeKind::StillLife));
        }
    }

    #[test]
    fn classifies_a_still_life_across_a_wrapped_edge() {
        let cells = [(19, 4), (0, 4), (19, 5), (0, 5)];
        let life = board_with(&cells);
        let labels = classify(&life);

        for (x, y) in cells {
            let x = usize::try_from(x).expect("test x coordinate is non-negative");
            let y = usize::try_from(y).expect("test y coordinate is non-negative");
            assert_eq!(labels[y * life.width() + x], Some(ShapeKind::StillLife));
        }
    }

    #[test]
    fn classifies_an_oscillator() {
        let life = board_with(&[(3, 4), (4, 4), (5, 4)]);
        let labels = classify(&life);
        assert_eq!(labels[4 * life.width() + 3], Some(ShapeKind::Oscillator));
    }

    #[test]
    fn classifies_a_glider_and_spaceship() {
        let glider = board_with(GLIDER);
        let glider_labels = classify(&glider);
        assert_eq!(glider_labels[2 * glider.width()], Some(ShapeKind::Glider));

        let ship = board_with(LIGHTWEIGHT_SPACESHIP);
        let ship_labels = classify(&ship);
        assert_eq!(ship_labels[ship.width()], Some(ShapeKind::Spaceship));
    }

    #[test]
    fn labels_every_cell_of_an_unrecognized_component() {
        let cells = [(4, 4), (5, 4), (6, 4), (4, 5), (5, 5)];
        let life = board_with(&cells);
        let labels = classify(&life);

        for (x, y) in cells {
            let x = usize::try_from(x).expect("test x coordinate is non-negative");
            let y = usize::try_from(y).expect("test y coordinate is non-negative");
            assert_eq!(labels[y * life.width() + x], Some(ShapeKind::Unrecognized));
        }
    }
}
