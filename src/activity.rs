use std::collections::VecDeque;

use crate::life::Life;

const MIN_GENERATIONS_BEFORE_QUIET_RESEED: u64 = 270;
const MAX_GENERATIONS_PER_SEED: u64 = 1_800;
const ACTIVITY_WINDOW_GENERATIONS: usize = 90;
const QUIET_ACTIVITY_PERMILLE: usize = 4;

/// Rolling generation-to-generation change data used to decide when a field
/// has become visually quiet.
#[derive(Debug, Default)]
pub struct ActivityMonitor {
    changes: VecDeque<usize>,
    total_changes: usize,
}

impl ActivityMonitor {
    /// Adds one generation's number of births and deaths to the rolling window.
    pub fn record(&mut self, change_count: usize) {
        self.changes.push_back(change_count);
        self.total_changes = self.total_changes.saturating_add(change_count);
        if self.changes.len() > ACTIVITY_WINDOW_GENERATIONS {
            self.total_changes = self
                .total_changes
                .saturating_sub(self.changes.pop_front().unwrap_or(0));
        }
    }

    /// Removes history after a manual or automatic re-seed.
    pub fn clear(&mut self) {
        self.changes.clear();
        self.total_changes = 0;
    }

    /// Returns true once the full window averages no more than 0.4% changed
    /// cells per generation. The absolute allowance grows with the actual
    /// visible board size, so a larger screen has a proportionally larger
    /// activity budget.
    #[must_use]
    pub fn is_quiet(&self, board_size: usize) -> bool {
        if self.changes.len() < ACTIVITY_WINDOW_GENERATIONS {
            return false;
        }
        let permitted_changes = board_size
            .saturating_mul(QUIET_ACTIVITY_PERMILLE)
            .saturating_mul(self.changes.len())
            / 1_000;
        self.total_changes <= permitted_changes
    }
}

/// Decides when the current seed should be replaced.
#[must_use]
pub fn should_reseed(life: &Life, activity: &ActivityMonitor) -> bool {
    life.alive_count() == 0
        || life.generation() >= MAX_GENERATIONS_PER_SEED
        || (life.generation() >= MIN_GENERATIONS_BEFORE_QUIET_RESEED
            && activity.is_quiet(life.width() * life.height()))
}

#[cfg(test)]
mod tests {
    use super::{ACTIVITY_WINDOW_GENERATIONS, ActivityMonitor};

    #[test]
    fn quiet_threshold_scales_with_board_size() {
        let mut quiet = ActivityMonitor::default();
        for _ in 0..ACTIVITY_WINDOW_GENERATIONS {
            quiet.record(4);
        }
        assert!(quiet.is_quiet(1_000));

        let mut active = ActivityMonitor::default();
        for _ in 0..ACTIVITY_WINDOW_GENERATIONS {
            active.record(5);
        }
        assert!(!active.is_quiet(1_000));
    }
}
