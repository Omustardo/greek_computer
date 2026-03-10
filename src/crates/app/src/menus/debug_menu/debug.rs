use crate::MyAppState;
use chrono::Duration;
use egui::Ui;

impl MyAppState {
    pub(crate) fn debug_menu_content(&mut self, ctx: &egui::Context, ui: &mut Ui) {
        self.show_tick_controls(ui);
        ui.separator();
        self.show_resolution(ctx, ui);
        ui.separator();
    }
    fn show_tick_controls(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(format!(
                "Total Time: {}",
                format_duration_stable(self.tick.time_elapsed_total)
            ));
            ui.separator();
            ui.label(format!("Total Ticks: {}", self.tick.ticks_processed_total));
        });
        ui.separator();
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.tick.ticks_active, "Enable Ticks").on_hover_text("Uncheck this to stop updates to the state of the application");
        });

        ui.separator();
        ui.horizontal(|ui| {
            // NOTE: Setting a target FPS is not well supported in egui / eframe. I gave it a good attempt:
            //   https://github.com/Omustardo/aeons_decay/commit/067dcbb028b5d23d20dfdbb85a3653419320e0da
            //   * `ctx.request_repaint_after` in the `update` function didn't work at all. It only requests,
            //     so I guess something was making it update more frequently.
            //   * sleep() in the `update` function worked, but wasn't great. It ended up with a significantly
            //     lower FPS than I requested.
            //   * Claude had lots of suggestions that didn't work. Hallucinating methods on `eframe::NativeOptions`.
            let result = ui.label(format!("FPS: {}", self.session.fps_counter.get_human_fps()));
            result.on_hover_text("Frames per Second");
        });

        ui.separator();
        ui.horizontal(|ui| {
            ui.label("Target Ticks per Second:");
            let mut tps = 1.0 / self.tick.target_tick_interval.as_seconds_f32();
            if ui
                .add(
                    egui::DragValue::new(&mut tps)
                        .range(1.0..=1000.0)
                        .speed(1.0)
                        .fixed_decimals(0),
                )
                .changed()
            {
                let interval_microseconds = (1_000_000.0 / tps) as i64;
                self.tick.target_tick_interval = chrono::Duration::microseconds(interval_microseconds);
            }
            ui.separator();
            ui.label("Quick Set:");
            let common_tps = [1, 10, 50, 100, 250, 500, 1000];
            for &tps in &common_tps {
                if ui.button(format!("{tps}")).highlight().clicked() {
                    let interval_seconds = 1.0 / tps as f32;
                    self.tick.target_tick_interval = chrono::Duration::milliseconds((interval_seconds * 1000.0) as i64);
                }
            }
        });
    }
    fn show_resolution(&mut self, ctx: &egui::Context, ui: &mut Ui) {
        let screen_size = ctx.input(egui::InputState::screen_rect);
        let width = screen_size.width();
        let height = screen_size.height();
        ui.label(format!("Resolution: [width:{width},height:{height}]"));
    }
}

fn format_duration_stable(duration: Duration) -> String {
    let total_ms = duration.num_milliseconds();

    if total_ms < 1000 {
        format!("{total_ms:>6}ms")
    } else if total_ms < 60_000 {
        format!("{:>6.1}s", duration.as_seconds_f64())
    } else {
        let mins = total_ms / 60_000;
        let secs = (total_ms % 60_000) as f64 / 1000.0;
        format!("{mins:>2}m{secs:04.1}s")
    }
}
