const HISTORY_BUFFER_SIZE: usize = 500;
const ADJUSTMENT_SPEED: f32 = 0.02;

pub struct AdaptiveThreshold {
    threshold: f32,
    target_fpr: f32,
    recent_scores: [f32; HISTORY_BUFFER_SIZE],
    recent_anomalies: [bool; HISTORY_BUFFER_SIZE],
    index: usize,
    filled: bool,
}

impl AdaptiveThreshold {
    pub const fn new(initial_threshold: f32, target_fpr: f32) -> Self {
        Self {
            threshold: initial_threshold,
            target_fpr,
            recent_scores: [0.0; HISTORY_BUFFER_SIZE],
            recent_anomalies: [false; HISTORY_BUFFER_SIZE],
            index: 0,
            filled: false,
        }
    }

    pub fn update(&mut self, score: f32, is_anomaly: bool) {
        self.recent_scores[self.index] = score;
        self.recent_anomalies[self.index] = is_anomaly;
        self.index = (self.index + 1) % HISTORY_BUFFER_SIZE;
        if self.index == 0 {
            self.filled = true;
        }
        if self.filled && self.index % 10 == 0 {
            self.adjust_threshold();
        }
    }

    fn adjust_threshold(&mut self) {
        let len = if self.filled { HISTORY_BUFFER_SIZE } else { self.index };
        if len == 0 { return; }
        let mut anomaly_count = 0;
        for i in 0..len {
            if self.recent_anomalies[i] {
                anomaly_count += 1;
            }
        }
        let current_fpr = anomaly_count as f32 / len as f32;
        if current_fpr > self.target_fpr {
            self.threshold += ADJUSTMENT_SPEED;
        } else if current_fpr < self.target_fpr * 0.3 {
            self.threshold -= ADJUSTMENT_SPEED * 0.5;
        }
        if self.threshold < 0.10 { self.threshold = 0.10; }
if self.threshold > 0.95 { self.threshold = 0.95; }
    }

    pub fn get_threshold(&self) -> f32 {
        self.threshold
    }
}