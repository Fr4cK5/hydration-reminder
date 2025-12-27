#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    fs,
    time::{Duration, Instant},
};

use eframe::{
    App,
    egui::{self, CentralPanel, Color32, IconData, TextStyle, ViewportBuilder},
};
use schemars::schema_for;

use crate::config::{Config, FSConfig};

const ICON_BYTES: &[u8] = include_bytes!("../assets/icon.png");
const WIN_SIZE: [f32; 2] = [265., 70.];
const CONFIG_FILE_NAME: &'static str = "hrconfig.json";

mod config;
mod colors {
    use eframe::egui::Color32;

    pub const BLUE: Color32 = Color32::from_rgb(105, 138, 185);
    pub const RED: Color32 = Color32::from_rgb(201, 79, 109);
    pub const MUTED: Color32 = Color32::from_additive_luminance(100);
}

mod utils {
    use std::time::Duration;

    pub fn to_string_mins_secs(duration: &Duration) -> String {
        let minutes = duration.as_secs() / 60;
        let seconds = duration.as_secs() % 60;

        if minutes <= 0 {
            return format!("{:0>2}", seconds);
        }
        format!("{:0>2}:{:0>2}", minutes, seconds)
    }
}

fn main() -> eframe::Result<()> {
    let icon = image::load_from_memory_with_format(&ICON_BYTES, image::ImageFormat::Png)
        .expect("Invalid image bytes!")
        .to_rgba8();

    let (icon_width, icon_height) = icon.dimensions();

    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_icon(IconData {
                rgba: icon.into_raw(),
                width: icon_width,
                height: icon_height,
            })
            .with_min_inner_size(WIN_SIZE)
            .with_inner_size(WIN_SIZE)
            .with_always_on_top(),
        ..Default::default()
    };

    let app = HydrationReminder::new();
    if app.config.is_default {
        fs::write(
            CONFIG_FILE_NAME,
            serde_json::to_string(&FSConfig::default())
                .expect("Default config serialization error."),
        )
        .expect("Config file-write error.");
    }

    if cfg!(debug_assertions) {
        let schema = schema_for!(FSConfig);
        fs::write(
            "schema.json",
            serde_json::to_string_pretty(&schema).expect("Schema serialization error."),
        )
        .expect("Schema file-write error.");
    }

    eframe::run_native(
        "Hydration Reminder",
        options,
        Box::new(|_cc| Ok(Box::new(app))),
    )
}

struct HydrationReminder {
    startup_time: Instant,
    last_check: Instant,
    initial_remind_time: Instant,
    has_been_reminded: bool,
    first_reminder: bool,
    config: Config,
}

impl HydrationReminder {
    fn hydrate(&mut self) {
        self.last_check = Instant::now();
        self.first_reminder = true;
        self.has_been_reminded = false;
    }

    pub fn new() -> Self {
        Self {
            startup_time: Instant::now(),
            last_check: Instant::now(),
            initial_remind_time: Instant::now(),
            has_been_reminded: false,
            first_reminder: false,
            config: Config::try_from_path(CONFIG_FILE_NAME).unwrap_or_default(),
        }
    }
}

impl App for HydrationReminder {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(Duration::from_secs(5));
        CentralPanel::default().show(ctx, |ui| {
            let is_hovering_viewport = ui.ui_contains_pointer();
            if is_hovering_viewport {
                ctx.request_repaint_after(Duration::from_secs(1));
            }
            ui.vertical_centered_justified(|ui| {
                ui.style_mut()
                    .text_styles
                    .get_mut(&TextStyle::Heading)
                    .map(|style| style.size = 48.);

                if self.config.time_parsing_failed
                    && Instant::now().duration_since(self.startup_time) < Duration::from_secs(10)
                {
                    ui.colored_label(Color32::RED, "Invalid reminder interval, using default");
                    ui.colored_label(Color32::RED, "Missing a duration suffix such as s, m or h?");
                    ui.colored_label(Color32::RED, "Example: 10m, 20m30s, 1h1m1s");
                } else if !self.first_reminder
                    || self.last_check.elapsed() > self.config.reminder_interval
                {
                    // For more attention grabbing flashing
                    ctx.request_repaint_after(Duration::from_secs(1));

                    if !self.has_been_reminded {
                        self.initial_remind_time = Instant::now();
                        self.has_been_reminded = true;
                    }

                    let should_change_color = Instant::now()
                        .duration_since(self.initial_remind_time)
                        .as_secs()
                        % 2
                        == 0;

                    ui.visuals_mut().override_text_color = Some(if should_change_color {
                        colors::BLUE
                    } else {
                        colors::RED
                    });

                    let text = if is_hovering_viewport {
                        &utils::to_string_mins_secs(
                            &Instant::now().duration_since(self.initial_remind_time),
                        )
                    } else {
                        "Hydrate 💧"
                    };

                    if ui.heading(text).clicked() {
                        self.hydrate();
                    }
                } else {
                    ui.visuals_mut().override_text_color = Some(colors::MUTED);

                    let text = if is_hovering_viewport {
                        &utils::to_string_mins_secs(&Instant::now().duration_since(self.last_check))
                    } else {
                        "Nice"
                    };
                    if ui.heading(text).clicked_by(egui::PointerButton::Secondary) {
                        self.hydrate();
                    }
                }
            });
        });
    }
}
