use std::collections::VecDeque;
use std::ops::Sub;

/// A tool to count occurrences of events over time. Call `update` whenever an event happens.
/// For the expected use-case of counting frames per second while rendering the game,
/// `update` should be called within each invocation of `eframe::App::update()`.
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, PartialEq)]
pub struct FpsCounter {
    last_frame_time: chrono::DateTime<chrono::Local>,
    frame_times: VecDeque<f32>,
    max_samples: usize,

    // The fields below are for human readable displays. The trouble with FPS is that it
    // can rapidly change between values (e.g. 159 and 160) which looks bad in a UI.
    // Cache it for a sub-second duration to avoid flickering.
    last_display_update: chrono::DateTime<chrono::Local>,
    cached_display_fps: f32,
    display_update_interval: chrono::TimeDelta,
}

impl Default for FpsCounter {
    fn default() -> Self {
        Self::new()
    }
}

impl FpsCounter {
    #[must_use]
    pub fn new() -> Self {
        let now = chrono::Local::now();
        Self {
            last_frame_time: now,
            frame_times: VecDeque::new(),
            max_samples: 120, // Keep last N frame times

            last_display_update: now,
            cached_display_fps: 0.0,
            display_update_interval: chrono::TimeDelta::milliseconds(100),
        }
    }

    /// Update the internal FPS count. This should be called once per frame.
    pub fn update(&mut self) {
        let now = chrono::Local::now();
        let delta = now.signed_duration_since(self.last_frame_time).as_seconds_f32();
        self.last_frame_time = now;

        self.frame_times.push_back(delta);
        if self.frame_times.len() > self.max_samples {
            self.frame_times.pop_front();
        }

        if now.sub(self.display_update_interval) > self.last_display_update {
            self.cached_display_fps = self.get_fps().round();
            self.last_display_update = now;
        }
    }

    fn get_fps(&self) -> f32 {
        if self.frame_times.is_empty() {
            return 0.0;
        }
        let avg_frame_time = self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32;
        1.0 / avg_frame_time
    }
    #[must_use]
    pub fn get_human_fps(&self) -> f32 {
        self.cached_display_fps
    }
    #[must_use]
    pub fn since_last_frame(&self) -> chrono::TimeDelta {
        chrono::Local::now() - self.last_frame_time
    }
}
