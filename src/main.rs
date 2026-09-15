mod activity;
mod life;
mod options;
mod render;
mod screen;
mod shapes;

use std::time::{SystemTime, UNIX_EPOCH};

use activity::{ActivityMonitor, should_reseed};
use life::{Life, Rng};
use macroquad::prelude::*;
use options::window_conf;

const STARTUP_INPUT_GRACE_SECONDS: f64 = 0.75;

#[macroquad::main(window_conf)]
async fn main() {
    let settings = options::parse();
    show_mouse(settings.windowed);

    let mut rng = Rng::new(seed());
    let mut life = seeded_life(&mut rng, settings.density);
    let mut shapes = shapes::classify(&life);
    let mut activity = ActivityMonitor::default();

    let mut speed = settings.speed;
    let mut paused = false;
    let mut elapsed = 0.0;
    let mut previous_pointer = pointer_position();
    // The Enter key from a terminal or a global-launcher chord can arrive at
    // the freshly-created fullscreen window. It is not an intent to dismiss
    // the screensaver, so ignore input during this short hand-off period.
    let accept_input_after = get_time() + STARTUP_INPUT_GRACE_SECONDS;

    loop {
        let mut needs_shape_classification = if screen::board_needs_resize(&life) {
            life = seeded_life(&mut rng, settings.density);
            activity.clear();
            true
        } else {
            false
        };

        if settings.interactive {
            if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::Q) {
                break;
            }
            if is_key_pressed(KeyCode::Space) {
                paused = !paused;
            }
            if is_key_pressed(KeyCode::R) {
                life.seed_random(&mut rng, settings.density);
                activity.clear();
                needs_shape_classification = true;
            }
            if is_key_pressed(KeyCode::Up) {
                speed = (speed * 1.25).min(60.0);
            }
            if is_key_pressed(KeyCode::Down) {
                speed = (speed / 1.25).max(0.5);
            }
        } else if get_time() >= accept_input_after && user_interacted(previous_pointer) {
            break;
        }
        previous_pointer = pointer_position();

        if !paused {
            elapsed += get_frame_time();
            let generation_time = 1.0 / speed;
            let mut catch_up_steps = 0;
            while elapsed >= generation_time && catch_up_steps < 4 {
                elapsed -= generation_time;
                life.tick();
                activity.record(life.last_change_count());
                catch_up_steps += 1;
                needs_shape_classification = true;
            }

            if should_reseed(&life, &activity) {
                life.seed_random(&mut rng, settings.density);
                activity.clear();
                needs_shape_classification = true;
            }
        }
        if needs_shape_classification {
            shapes = shapes::classify(&life);
        }

        render::draw(&life, &shapes);
        next_frame().await;
    }

    show_mouse(true);
}

fn seeded_life(rng: &mut Rng, density: f32) -> Life {
    let (columns, rows) = screen::grid_dimensions();
    let mut life = Life::new(columns, rows);
    life.seed_random(rng, density);
    life
}

fn seed() -> u64 {
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| {
            u64::try_from(duration.as_nanos()).unwrap_or(u64::MAX)
        });
    time ^ u64::from(std::process::id()).rotate_left(21)
}

fn pointer_position() -> Vec2 {
    let (x, y) = mouse_position();
    vec2(x, y)
}

fn user_interacted(previous_pointer: Vec2) -> bool {
    get_last_key_pressed().is_some()
        || is_mouse_button_pressed(MouseButton::Left)
        || is_mouse_button_pressed(MouseButton::Right)
        || is_mouse_button_pressed(MouseButton::Middle)
        || mouse_wheel() != (0.0, 0.0)
        || pointer_position().distance(previous_pointer) > 3.0
}
