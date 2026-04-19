//! Эксперт одного сигнала. Финальная версия: "Сравнение Волатильностей" + "Мертвая Зона".
//! Радостная версия для копипаста.

use libm::sqrtf;

const LONG_WINDOW_SIZE: usize = 400;
const SHORT_WINDOW_SIZE: usize = 30; // Это значение используется только для коэффициента alpha
const MIN_STD: f32 = 0.01;
const TARGET_FPR_PERCENT: f32 = 3.0;
const INITIAL_THRESHOLD: f32 = 0.7;

// --- Адаптивный порог ---
mod adaptive_threshold {
    pub struct AdaptiveThreshold {
        value: f32,
        target_fpr: f32,
        ema_fpr: f32,
        anomalies: [bool; 100],
        idx: usize,
        filled: bool,
    }
    impl AdaptiveThreshold {
        pub const fn new(initial: f32, target_fpr_percent: f32) -> Self {
            Self {
                value: initial,
                target_fpr: target_fpr_percent / 100.0,
                ema_fpr: target_fpr_percent / 100.0,
                anomalies: [false; 100],
                idx: 0,
                filled: false,
            }
        }
        pub fn update(&mut self, is_anomaly: bool) {
            self.anomalies[self.idx] = is_anomaly;
            self.idx = (self.idx + 1) % 100;
            if self.idx == 0 {
                self.filled = true;
            }
            if self.filled && self.idx % 10 == 0 {
                self.adjust();
            }
        }
        fn adjust(&mut self) {
            let anomaly_count = self.anomalies.iter().filter(|&&x| x).count();
            let current_fpr = anomaly_count as f32 / 100.0;
            self.ema_fpr = self.ema_fpr * 0.9 + current_fpr * 0.1;
            let error = self.ema_fpr - self.target_fpr;
            self.value += error * 0.1;
            self.value = self.value.clamp(0.20, 0.80);
        }
        pub fn get(&self) -> f32 {
            self.value
        }
    }
}

use adaptive_threshold::AdaptiveThreshold;

pub struct SignalExpert {
    long_term_mean: f32,
    long_term_std: f32,
    short_term_mean: f32,
    short_term_sq_diff: f32,
    training_history: [f32; LONG_WINDOW_SIZE],
    hist_idx: usize,
    threshold: AdaptiveThreshold,
    training: bool,
}

impl SignalExpert {
    pub const fn new() -> Self {
        Self {
            long_term_mean: 0.0,
            long_term_std: 1.0,
            short_term_mean: 0.0,
            short_term_sq_diff: 0.0,
            training_history: [0.0; LONG_WINDOW_SIZE],
            hist_idx: 0,
            threshold: AdaptiveThreshold::new(INITIAL_THRESHOLD, TARGET_FPR_PERCENT),
            training: true,
        }
    }

    pub fn process(&mut self, value: f32) -> f32 {
        if self.training {
            self.training_history[self.hist_idx] = value;
            self.hist_idx += 1;
            if self.hist_idx >= LONG_WINDOW_SIZE {
                self.finish_training(value);
            }
            return 0.0;
        }
        let alpha: f32 = 2.0 / (SHORT_WINDOW_SIZE as f32 + 1.0);
        self.short_term_mean = self.short_term_mean * (1.0 - alpha) + value * alpha;
        let sq_diff = (value - self.short_term_mean) * (value - self.short_term_mean);
        self.short_term_sq_diff = self.short_term_sq_diff * (1.0 - alpha) + sq_diff * alpha;
        let score = self.compute_anomaly_score(value);
        if score < 0.1 {
            self.adapt_long_term_model(value);
        }
        let is_anomaly = score > self.threshold.get();
        self.threshold.update(is_anomaly);
        score
    }
    
    pub fn process_learning(&mut self, value: f32) {
        if self.training {
            self.training_history[self.hist_idx] = value;
            self.hist_idx += 1;
            if self.hist_idx >= LONG_WINDOW_SIZE {
                self.finish_training(value);
            }
        }
    }

    pub fn get_score(&mut self, value: f32){
        self.score
    }

    fn finish_training(&mut self, _last_value: f32) {
        let sum: f32 = self.training_history.iter().sum();
        self.long_term_mean = sum / LONG_WINDOW_SIZE as f32;
        let mut sq_sum = 0.0;
        for &v in &self.training_history {
            sq_sum += (v - self.long_term_mean) * (v - self.long_term_mean);
        }
        self.long_term_std = sqrtf(sq_sum / LONG_WINDOW_SIZE as f32).max(MIN_STD);
        self.short_term_mean = self.long_term_mean;
        self.short_term_sq_diff = self.long_term_std * self.long_term_std;
        self.training = false;
    }

    fn compute_anomaly_score(&self, value: f32) -> f32 {
        let short_term_std = sqrtf(self.short_term_sq_diff).max(MIN_STD);
        let ratio = short_term_std / self.long_term_std;

        // --- "МЕРТВАЯ ЗОНА" ДЛЯ ВОЛАТИЛЬНОСТИ ---
        const DEAD_ZONE_UPPER: f32 = 1.4;
        const DEAD_ZONE_LOWER: f32 = 0.6;

        let vol_score = if ratio > DEAD_ZONE_UPPER {
            (ratio - DEAD_ZONE_UPPER) / 2.0
        } else if ratio < DEAD_ZONE_LOWER {
            (DEAD_ZONE_LOWER - ratio) / DEAD_ZONE_LOWER
        } else {
            0.0
        };

        // --- Z-Score для дрейфа и одиночных выбросов ---
        let z_score = ((value - self.long_term_mean) / self.long_term_std).abs();
        let z_score_norm = if z_score > 2.5 {
            (z_score - 2.5) / 3.0
        } else {
            0.0
        };
        
        vol_score.max(z_score_norm).clamp(0.0, 1.0)
    }

    fn adapt_long_term_model(&mut self, value: f32) {
        self.long_term_mean = self.long_term_mean * 0.9999 + value * 0.0001;
        let current_sq_dev = (value - self.long_term_mean) * (value - self.long_term_mean);
        let new_var = (self.long_term_std * self.long_term_std) * 0.99995 + current_sq_dev * 0.00005;
        self.long_term_std = sqrtf(new_var).max(MIN_STD);
    }

    pub fn get_threshold(&self) -> f32 {
        if self.training { 1.0 } else { self.threshold.get() }
    }

    pub fn reset(&mut self) { *self = Self::new(); }
}

impl Default for SignalExpert {
    fn default() -> Self {
        Self::new()
    }
}
