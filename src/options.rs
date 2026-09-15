use macroquad::prelude::Conf;

const DEFAULT_DENSITY: f32 = 0.16;
const DEFAULT_SPEED: f32 = 9.0;

/// Command-line options shared by the normal screensaver and preview modes.
#[derive(Debug, Clone, Copy)]
pub struct Options {
    pub interactive: bool,
    pub windowed: bool,
    pub density: f32,
    pub speed: f32,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            interactive: false,
            windowed: false,
            density: DEFAULT_DENSITY,
            speed: DEFAULT_SPEED,
        }
    }
}

/// Parses `LifeSaver`'s deliberately small command-line surface.
#[must_use]
pub fn parse() -> Options {
    let mut options = Options::default();
    let mut arguments = std::env::args().skip(1);
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--interactive" | "-i" => options.interactive = true,
            "--windowed" | "-w" => options.windowed = true,
            "--density" => {
                if let Some(value) = arguments.next().and_then(|value| value.parse().ok()) {
                    options.density = value;
                }
            }
            "--speed" => {
                if let Some(value) = arguments.next().and_then(|value| value.parse().ok()) {
                    options.speed = value;
                }
            }
            _ => {}
        }
    }
    options.density = options.density.clamp(0.01, 0.95);
    options.speed = options.speed.clamp(0.5, 60.0);
    options
}

/// Configures Macroquad before its event loop starts.
#[must_use]
pub fn window_conf() -> Conf {
    let options = parse();
    Conf {
        window_title: "LifeSaver".into(),
        window_width: 1280,
        window_height: 800,
        window_resizable: true,
        fullscreen: !options.windowed,
        high_dpi: true,
        ..Default::default()
    }
}
