//! Эксперт одного сигнала. Финальная версия: "Сравнение Волатильностей".
//! Устойчив к шуму, вычислительно эффективен.

use libm::sqrtf;

const LONG_WINDOW_SIZE: usize = 400;
const SHORT_WINDOW_SIZE: usize = 30; // Это значение используется только для коэффициента alpha
const MIN_STD: f32 = 0.01;
const TARGET_FPR_PERCENT: f32 = 3.0; // Более агрессивная цель
const INITIAL_THRESHOLD: f32 = 0.5;

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
            // Ускоряем адаптацию, чтобы быстрее сходиться к цели
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
    // Долгосрочная модель
    long_term_mean: f32,
    long_term_std: f32,

    // Онлайн-статистика для короткого окна (O(1) сложность)
    short_term_mean: f32,
    short_term_sq_diff: f32, // EMA квадрата разницы для вычисления дисперсии
    
    // История для обучения
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

        // 1. Обновляем онлайн-статистику короткого окна (EMA)
        let alpha: f32 = 2.0 / (SHORT_WINDOW_SIZE as f32 + 1.0); // ~0.06
        self.short_term_mean = self.short_term_mean * (1.0 - alpha) + value * alpha;
        let sq_diff = (value - self.short_term_mean) * (value - self.short_term_mean);
        self.short_term_sq_diff = self.short_term_sq_diff * (1.0 - alpha) + sq_diff * alpha;

        // 2. Вычисляем score
        let score = self.compute_anomaly_score(value);

        // 3. Медленно адаптируем глобальную модель
        if score < 0.1 {
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
            sq_sum += (v - self.long_term_mean) * (v - self.long_term_mean);
        }
        self.long_term_std = sqrtf(sq_sum / LONG_WINDOW_SIZE as f32).max(MIN_STD);
        
        // Инициализируем онлайн-статистику на основе долгосрочной
        self.short_term_mean = self.long_term_mean;
        self.short_term_sq_diff = self.long_term_std * self.long_term_std;
        
        self.training = false;
    }

    fn compute_anomaly_score(&self, value: f32) -> f32 {
        let short_term_std = sqrtf(self.short_term_sq_diff).max(MIN_STD);

        // --- Основной score: отношение стандартных отклонений ---
        // Если текущая волатильность сильно выше или ниже нормы - это аномалия
        let ratio = short_term_std / self.long_term_std;
        let vol_score = if ratio > 1.0 {
            // Текущая волатильность выше нормы (шум, выброс)
            (ratio - 1.0) / 3.0 // ratio=4 -> score=1.0
        } else {
            // Текущая волатильность ниже нормы (заморозка)
            (1.0 - ratio) / 0.9 // ratio=0.1 -> score=1.0
        };

        // --- Дополнительный score: Z-оценка для ловли единичных выбросов и дрейфа ---
        let z_score = ((value - self.long_term_mean) / self.long_term_std).abs();
        let z_score_norm = (z_score / 5.0).clamp(0.0, 1.0); // z=5 -> 1.0

        // Итоговый score - это максимум из двух подходов
        vol_score.max(z_score_norm).clamp(0.0, 1.0)
    }

    fn adapt_long_term_model(&mut self, value: f32) {
        self.long_term_mean = self.long_term_mean * 0.9999 + value * 0.0001;
        let current_sq_dev = (value - self.long_term_mean) * (value - self.long_term_mean);
        let new_var = (self.long_term_std * self.long_term_std) * 0.99995 + current_sq_dev * 0.00005;
        self.long_term_std = sqrtf(new_var).max(MIN_STD);
    }

    pub fn get_threshold(&self) -> f32 {
        if self.training {
            1.0
        } else {
            self.threshold.get()
        }
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

impl Default for SignalExpert {
    fn default() -> Self {
        Self::new()
    }
}
