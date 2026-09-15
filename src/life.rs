//! The dependency-free Conway's Game of Life engine used by `LifeSaver`.

/// A compact, toroidal Conway's Game of Life board.
///
/// Coordinates wrap at every edge, so gliders and other moving patterns do
/// not disappear when they reach the screen boundary.
#[derive(Debug, Clone)]
pub struct Life {
    width: usize,
    height: usize,
    cells: Vec<Cell>,
    generation: u64,
    last_changed: bool,
    last_change_count: usize,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct Cell {
    alive: bool,
    age: u16,
}

impl Life {
    /// Creates an empty board. Both dimensions must be non-zero and fit in an
    /// [`isize`], which makes wrapped coordinate arithmetic well-defined.
    ///
    /// # Panics
    ///
    /// Panics if either dimension is zero, cannot fit in an [`isize`], or if
    /// the requested cell count would overflow [`usize`].
    #[must_use]
    pub fn new(width: usize, height: usize) -> Self {
        assert!(width > 0, "a Life board needs a non-zero width");
        assert!(height > 0, "a Life board needs a non-zero height");
        assert!(isize::try_from(width).is_ok(), "board width is too large");
        assert!(isize::try_from(height).is_ok(), "board height is too large");
        let cell_count = width
            .checked_mul(height)
            .expect("Life board cell count overflowed usize");

        Self {
            width,
            height,
            cells: vec![Cell::default(); cell_count],
            generation: 0,
            last_changed: false,
            last_change_count: 0,
        }
    }

    /// Board width in cells.
    #[must_use]
    pub const fn width(&self) -> usize {
        self.width
    }

    /// Board height in cells.
    #[must_use]
    pub const fn height(&self) -> usize {
        self.height
    }

    /// The number of generations since the last seed.
    #[must_use]
    pub const fn generation(&self) -> u64 {
        self.generation
    }

    /// The number of cells that changed state in the previous generation.
    #[must_use]
    pub const fn last_change_count(&self) -> usize {
        self.last_change_count
    }

    /// Returns whether a cell is alive. Coordinates wrap around the board.
    #[must_use]
    pub fn is_alive(&self, x: isize, y: isize) -> bool {
        self.cells[self.index_wrapped(x, y)].alive
    }

    /// Returns whether the in-bounds cell is alive.
    #[must_use]
    pub fn is_alive_at(&self, x: usize, y: usize) -> bool {
        self.cells[self.index(x, y)].alive
    }

    /// Returns the age of a live cell; dead cells always have age zero.
    #[must_use]
    pub fn age(&self, x: usize, y: usize) -> u16 {
        self.cells[self.index(x, y)].age
    }

    /// Sets a cell's state for deterministic pattern tests.
    #[cfg(test)]
    pub fn set_alive(&mut self, x: usize, y: usize, alive: bool) {
        let index = self.index(x, y);
        let cell = &mut self.cells[index];
        cell.alive = alive;
        cell.age = u16::from(alive);
    }

    /// Seeds the whole board using a small deterministic random-number
    /// generator. `density` is clamped to the inclusive 0.0–1.0 range.
    pub fn seed_random(&mut self, rng: &mut Rng, density: f32) {
        let density = density.clamp(0.0, 1.0);
        for cell in &mut self.cells {
            cell.alive = rng.next_unit() < density;
            cell.age = u16::from(cell.alive);
        }
        self.generation = 0;
        self.last_changed = true;
        self.last_change_count = 0;
    }

    /// Counts currently live cells.
    #[must_use]
    pub fn alive_count(&self) -> usize {
        self.cells.iter().filter(|cell| cell.alive).count()
    }

    /// Advances the board by one generation, returning whether its layout
    /// changed. Conway's classic B3/S23 rules are applied.
    pub fn tick(&mut self) -> bool {
        let mut next = Vec::with_capacity(self.cells.len());
        let mut change_count = 0;

        for y in 0..self.height {
            for x in 0..self.width {
                let current = self.cells[self.index(x, y)];
                let neighbours = self.live_neighbour_count(x, y);
                let alive = neighbours == 3 || (current.alive && neighbours == 2);
                change_count += usize::from(alive != current.alive);
                next.push(Cell {
                    alive,
                    age: if alive {
                        if current.alive {
                            current.age.saturating_add(1)
                        } else {
                            1
                        }
                    } else {
                        0
                    },
                });
            }
        }

        self.cells = next;
        self.generation = self.generation.saturating_add(1);
        self.last_changed = change_count > 0;
        self.last_change_count = change_count;
        self.last_changed
    }

    fn index(&self, x: usize, y: usize) -> usize {
        assert!(
            x < self.width && y < self.height,
            "cell coordinate is outside the board"
        );
        y * self.width + x
    }

    fn index_wrapped(&self, x: isize, y: isize) -> usize {
        let width =
            isize::try_from(self.width).expect("board dimensions were validated at creation");
        let height =
            isize::try_from(self.height).expect("board dimensions were validated at creation");
        let wrapped_x =
            usize::try_from(x.rem_euclid(width)).expect("wrapped x coordinate is non-negative");
        let wrapped_y =
            usize::try_from(y.rem_euclid(height)).expect("wrapped y coordinate is non-negative");
        self.index(wrapped_x, wrapped_y)
    }

    fn live_neighbour_count(&self, x: usize, y: usize) -> u8 {
        let x = isize::try_from(x).expect("board dimensions were validated at creation");
        let y = isize::try_from(y).expect("board dimensions were validated at creation");
        let mut count = 0;
        for offset_y in -1..=1 {
            for offset_x in -1..=1 {
                if offset_x != 0 || offset_y != 0 {
                    count += u8::from(self.is_alive(x + offset_x, y + offset_y));
                }
            }
        }
        count
    }
}

/// Tiny xorshift generator: reproducible, dependency-free, and entirely
/// sufficient for scattering initial Life cells.
#[derive(Debug, Clone, Copy)]
pub struct Rng(u64);

impl Rng {
    /// Constructs a generator. A zero seed is adjusted because xorshift's
    /// all-zero internal state cannot advance.
    #[must_use]
    pub const fn new(seed: u64) -> Self {
        Self(if seed == 0 {
            0x9e37_79b9_7f4a_7c15
        } else {
            seed
        })
    }

    /// Produces a uniformly distributed number in `[0.0, 1.0)`.
    pub fn next_unit(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        let mantissa = u32::try_from(self.0 >> 41).unwrap_or(0);
        f32::from_bits(0x3f80_0000 | mantissa) - 1.0
    }
}

#[cfg(test)]
mod tests {
    use super::Life;

    #[test]
    fn block_is_still_life() {
        let mut life = Life::new(6, 6);
        for (x, y) in [(2, 2), (3, 2), (2, 3), (3, 3)] {
            life.set_alive(x, y, true);
        }

        assert!(!life.tick());
        assert_eq!(life.alive_count(), 4);
        for (x, y) in [(2, 2), (3, 2), (2, 3), (3, 3)] {
            assert!(life.is_alive(x, y));
        }
    }

    #[test]
    fn blinker_alternates_orientation() {
        let mut life = Life::new(7, 7);
        for y in 2..=4 {
            life.set_alive(3, y, true);
        }

        assert!(life.tick());
        for x in 2..=4 {
            assert!(life.is_alive(x, 3));
        }
        assert!(life.tick());
        for y in 2..=4 {
            assert!(life.is_alive(3, y));
        }
    }

    #[test]
    fn glider_moves_diagonally_after_four_generations() {
        let mut life = Life::new(12, 12);
        for (x, y) in [(1, 0), (2, 1), (0, 2), (1, 2), (2, 2)] {
            life.set_alive(x, y, true);
        }
        for _ in 0..4 {
            life.tick();
        }

        for (x, y) in [(2, 1), (3, 2), (1, 3), (2, 3), (3, 3)] {
            assert!(life.is_alive(x, y), "expected a live cell at ({x}, {y})");
        }
    }

    #[test]
    fn edges_wrap() {
        let mut life = Life::new(5, 5);
        for (x, y) in [(4, 2), (0, 2), (1, 2)] {
            life.set_alive(x, y, true);
        }

        life.tick();
        assert!(life.is_alive(0, 1));
        assert!(life.is_alive(0, 2));
        assert!(life.is_alive(0, 3));
    }
}
