//! Эксперт одного сигнала. Финальная версия: "Два Окна".
//! Максимально упрощенная и надежная логика для борьбы с FPR.

use libm::sqrtf;

const LONG_WINDOW_SIZE: usize = 400; // Окно для глобальной "нормальной" модели
const SHORT_WINDOW_SIZE: usize = 30; // Окно для локальной, быстрой "нормы"
const MIN_STD: f32 = 0.01;
const TARGET_FPR_PERCENT: f32 = 4.0; // Сделаем цель чуть ниже
const INITIAL_THRESHOLD: f32 = 0.5;  // И порог чуть выше

// --- Адаптивный порог ---
mod adaptive_threshold {
    pub struct AdaptiveThreshold {
        value: f32, target_fpr: f32, ema_fpr: f32,
        anomalies: [bool; 100], idx: usize, filled: bool,
    }
    impl AdaptiveThreshold {
        pub const fn new(initial: f32, target_fpr_percent: f32) -> Self {
            Self {
                value: initial, target_fpr: target_fpr_percent / 100.0,
                ema_fpr: target_fpr_percent / 100.0, anomalies: [false; 100],
                idx: 0, filled: false,
            }
        }
        pub fn update(&mut self, is_anomaly: bool) {
            self.anomalies[self.idx] = is_anomaly;
            self.idx = (self.idx + 1) % 100;
            if self.idx == 0 { self.filled = true; }
            if self.filled && self.idx % 10 == 0 { self.adjust(); }
        }
        fn adjust(&mut self) {
            let anomaly_count = self.anomalies.iter().filter(|&&x| x).count();
            let current_fpr = anomaly_count as f32 / 100.0;
            self.ema_fpr = self.ema_fpr * 0.9 + current_fpr * 0.1;
            let error = self.ema_fpr - self.target_fpr;
            self.value += error * 0.05;
            self.value = self.value.clamp(0.30, 0.80);
        }
        pub fn get(&self) -> f32 { self.value }
    }
}

use adaptive_threshold::AdaptiveThreshold;

pub struct SignalExpert {
    // Глобальная "нормальная" модель (медленная)
    long_term_mean: f32,
    long_term_std: f32,

    // Локальная модель "текущего" состояния (быстрая)
    short_term_history: [f32; SHORT_WINDOW_SIZE],
    
    // История для первоначального обучения
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
            short_term_history: [0.0; SHORT_WINDOW_SIZE],
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

        // 1. Обновляем историю для локального окна
        self.short_term_history.copy_within(0..SHORT_WINDOW_SIZE - 1, 1);
        self.short_term_history[0] = value;

        // 2. Вычисляем score
        let score = self.compute_anomaly_score(value);

        // 3. Медленно адаптируем глобальную модель, если сигнал "спокоен"
        if score < 0.2 {
            self.adapt_long_term_model(value);
        }

        // 4. Обновляем порог
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

    fn finish_training(&mut self, last_value: f32) {
        let sum: f32 = self.training_history.iter().sum();
        self.long_term_mean = sum / LONG_WINDOW_SIZE as f32;
        
        let mut sq_sum = 0.0;
        for &v in &self.training_history {
            let diff = v - self.long_term_mean;
            sq_sum += diff * diff;
        }
        self.long_term_std = sqrtf(sq_sum / LONG_WINDOW_SIZE as f32).max(MIN_STD);
        
        self.short_term_history = [last_value; SHORT_WINDOW_SIZE];
        
        self.training = false;
    }

    fn compute_anomaly_score(&self, value: f32) -> f32 {
        // --- Вычисляем статистику для КОРОТКОГО окна ---
        let short_sum: f32 = self.short_term_history.iter().sum();
        let short_mean = short_sum / SHORT_WINDOW_SIZE as f32;
        let mut short_sq_sum = 0.0;
        for &v in &self.short_term_history {
            short_sq_sum += (v - short_mean) * (v - short_mean);
        }
        let short_std = sqrtf(short_sq_sum / SHORT_WINDOW_SIZE as f32).max(MIN_STD);

        // --- Вычисляем ДВЕ Z-оценки ---
        let z_long = ((value - self.long_term_mean) / self.long_term_std).abs();
        let z_short = ((value - short_mean) / short_std).abs();

        // --- ЛОГИКА СКОРИНГА ---
        let base_score = (z_short / 4.0).clamp(0.0, 1.0); // z=4 -> 1.0
        let drift_bonus = (z_long / 10.0).clamp(0.0, 0.2); // Максимум +0.2 к score

        (base_score + drift_bonus).clamp(0.0, 1.0)
    }

    fn adapt_long_term_model(&mut self, value: f32) {
        self.long_term_mean = self.long_term_mean * 0.9998 + value * 0.0002;
        let current_sq_dev = (value - self.long_term_mean) * (value - self.long_term_mean);
        let new_var = (self.long_term_std * self.long_term_std) * 0.9999 + current_sq_dev * 0.0001;
        self.long_term_std = sqrtf(new_var).max(MIN_STD);
    }

    pub fn get_threshold(&self) -> f32 {
        if self.training { 1.0 } else { self.threshold.get() }
    }

    pub fn reset(&mut self) { *self = Self::new(); }
}

impl Default for SignalExpert {
    fn default() -> Self { Self::new() }
}
