use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

pub struct FrameRateLimit {
    frames_to_track: usize,
    frame_starts: VecDeque<Instant>,
    target_duration: Duration,
}

impl FrameRateLimit {
    pub fn new(target_fps: u32, frames_to_track: usize) -> Self {
        Self {
            frames_to_track,
            frame_starts: VecDeque::with_capacity(frames_to_track),
            target_duration: Duration::from_secs(1) / target_fps,
        }
    }

    pub fn set_target_fps(&mut self, target_fps: u32) {
        self.target_duration = Duration::from_secs(1) / target_fps;
    }

    pub fn start_frame(&mut self) {
        if self.frame_starts.len() > self.frames_to_track {
            self.frame_starts.pop_back();
        }
        self.frame_starts.push_front(Instant::now());
    }

    pub fn sleep_to_limit(&self) {
        let elapsed = Instant::now() - *self.frame_starts.front().unwrap();
        if elapsed < self.target_duration {
            let remaining = self.target_duration - elapsed;
            self.precise_sleep(remaining);
        }
    }

    fn precise_sleep(&self, duration: Duration) {
        let start = Instant::now();
        while start.elapsed() < duration {
            std::hint::spin_loop();
        }
    }

    pub fn avg_frame_time(&self) -> Duration {
        let oldest_frame = self.frame_starts.back().unwrap();
        let total_duration = Instant::now() - *oldest_frame;
        total_duration / self.frame_starts.len() as u32
    }
}
