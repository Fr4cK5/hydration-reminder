#![windows_subsystem = "windows"]
use std::time::{Duration, Instant};

use eframe::{
    App,
    egui::{self, CentralPanel, IconData, TextStyle, ViewportBuilder},
};

#[cfg(not(debug_assertions))]
const DRINK_INTERVAL: Duration = Duration::from_secs(20 * 60); // 20 mins

#[cfg(debug_assertions)]
const DRINK_INTERVAL: Duration = Duration::from_secs(1);

const ICON_BYTES: &[u8] = include_bytes!("../assets/icon.png");
const WIN_SIZE: [f32; 2] = [265., 70.];

mod colors {
    use eframe::egui::Color32;

    pub const BLUE: Color32 = Color32::from_rgb(105, 138, 185);
    pub const RED: Color32 = Color32::from_rgb(201, 79, 109);
    pub const MUTED: Color32 = Color32::from_additive_luminance(100);
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

    eframe::run_native(
        "Hydration Reminder",
        options,
        Box::new(|_cc| Ok(Box::new(HydrationReminder::default()))),
    )
}

struct HydrationReminder {
    last_check: Instant,
    initial_remind_time: Instant,
    has_been_reminded: bool,
    first_reminder: bool,
}

impl Default for HydrationReminder {
    fn default() -> Self {
        Self {
            last_check: Instant::now(),
            initial_remind_time: Instant::now(),
            has_been_reminded: false,
            first_reminder: false,
        }
    }
}

impl App for HydrationReminder {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(Duration::from_secs(5));
        CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| {
                ui.style_mut()
                    .text_styles
                    .get_mut(&TextStyle::Heading)
                    .map(|style| style.size = 48.);

                if !self.first_reminder || self.last_check.elapsed() > DRINK_INTERVAL {
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

                    if ui.heading("Hydrate 💧").clicked() {
                        self.last_check = Instant::now();
                        self.first_reminder = true;
                        self.has_been_reminded = false;
                    }
                } else {
                    ui.visuals_mut().override_text_color = Some(colors::MUTED);
                    ui.heading("Nice");
                }
            });
        });
    }
}
