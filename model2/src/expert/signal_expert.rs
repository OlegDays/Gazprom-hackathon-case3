//! Эксперт одного сигнала с адаптивным порогом и защитой от загрязнения

use libm::sqrtf;

const WINDOW_SIZE: usize = 200;
const ZSCORE_THRESHOLD: f32 = 3.5;        // Выше порог z-score
const UPDATE_INTERVAL: usize = 20;
const MIN_STD: f32 = 0.01;
const TARGET_FPR: f32 = 0.05;             // Целевой FPR 5%

pub struct SignalExpert {
    // История для статистики
    history: [f32; WINDOW_SIZE],
    hist_idx: usize,
    hist_filled: bool,
    
    // Защищённая модель нормального поведения
    normal_mean: f32,
    normal_std: f32,
    normal_min: f32,
    normal_max: f32,
    
    // Текущая статистика
    current_mean: f32,
    current_std: f32,
    
    // Адаптивный порог
    threshold: f32,
    
    // Для адаптации порога
    recent_scores: [f32; 100],
    recent_anomalies: [bool; 100],
    recent_idx: usize,
    recent_filled: bool,
    
    processed: usize,
    training: bool,
    train_count: usize,
}

impl SignalExpert {
    pub fn new() -> Self {
        Self {
            history: [0.0; WINDOW_SIZE],
            hist_idx: 0,
            hist_filled: false,
            normal_mean: 0.0,
            normal_std: 1.0,
            normal_min: f32::MAX,
            normal_max: f32::MIN,
            current_mean: 0.0,
            current_std: 1.0,
            threshold: 0.75,                // Стартовый порог выше
            recent_scores: [0.0; 100],
            recent_anomalies: [false; 100],
            recent_idx: 0,
            recent_filled: false,
            processed: 0,
            training: true,
            train_count: 0,
        }
    }
    
    pub fn process(&mut self, value: f32) -> f32 {
        self.processed += 1;
        
        // Обучение на первых WINDOW_SIZE точках
        if self.training {
            self.history[self.train_count] = value;
            self.train_count += 1;
            if self.train_count >= WINDOW_SIZE {
                self.finish_training();
            }
            return 0.0;
        }
        
        // Скользящее окно для текущей статистики
        self.history[self.hist_idx] = value;
        self.hist_idx = (self.hist_idx + 1) % WINDOW_SIZE;
        
        if self.processed % UPDATE_INTERVAL == 0 {
            self.update_current_stats();
        }
        
        let score = self.compute_anomaly_score(value);
        
        // Очень медленная адаптация нормальной модели при score < 0.2
        if score < 0.2 {
            self.adapt_normal_model(value);
        }
        
        // Адаптация порога
        let is_anomaly = score > self.threshold;
        self.update_threshold(score, is_anomaly);
        
        score
    }
    
    pub fn process_learning(&mut self, value: f32) {
        if self.training {
            self.history[self.train_count] = value;
            self.train_count += 1;
            if self.train_count >= WINDOW_SIZE {
                self.finish_training();
            }
        }
    }
    
    fn finish_training(&mut self) {
        let sum: f32 = self.history.iter().sum();
        self.normal_mean = sum / WINDOW_SIZE as f32;
        
        let mut sq_sum = 0.0;
        for &v in &self.history {
            let diff = v - self.normal_mean;
            sq_sum += diff * diff;
            if v < self.normal_min { self.normal_min = v; }
            if v > self.normal_max { self.normal_max = v; }
        }
        self.normal_std = sqrtf(sq_sum / WINDOW_SIZE as f32).max(MIN_STD);
        
        self.current_mean = self.normal_mean;
        self.current_std = self.normal_std;
        
        self.training = false;
        self.hist_filled = true;
    }
    
    fn update_current_stats(&mut self) {
        let sum: f32 = self.history.iter().sum();
        self.current_mean = sum / WINDOW_SIZE as f32;
        
        let mut sq_sum = 0.0;
        for &v in &self.history {
            let diff = v - self.current_mean;
            sq_sum += diff * diff;
        }
        self.current_std = sqrtf(sq_sum / WINDOW_SIZE as f32).max(MIN_STD);
    }
    
    fn compute_anomaly_score(&self, value: f32) -> f32 {
        let z_normal = (value - self.normal_mean).abs() / self.normal_std;
        let z_current = (value - self.current_mean).abs() / self.current_std;
        let normal_range = self.normal_max - self.normal_min;
        
        let mut score = 0.0f32;
        
        // 1. Выход за нормальный диапазон
        if value < self.normal_min - normal_range * 0.3 
            || value > self.normal_max + normal_range * 0.3 {
            score = score.max(0.7);
        }
        
        // 2. Большой z-score по нормальной модели
        if z_normal > ZSCORE_THRESHOLD {
            score = score.max(0.6 + (z_normal - ZSCORE_THRESHOLD) * 0.1);
        }
        
        // 3. Подтверждение от текущего окна
        if z_current > 3.0 {
            score = score.max(0.8);
        }
        
        // 4. Заморозка
        if self.current_std < self.normal_std * 0.1 {
            score = score.max(0.85);
        }
        
        // 5. Штраф за частые колебания (шум) — снижаем скор, если std большой, но значение близко к среднему
        if z_normal < 1.0 && self.current_std > self.normal_std * 1.5 {
            score *= 0.7;
        }
        
        score.min(1.0)
    }
    
    fn adapt_normal_model(&mut self, value: f32) {
        // EMA с маленьким коэффициентом
        self.normal_mean = self.normal_mean * 0.995 + value * 0.005;
        
        if value < self.normal_min {
            self.normal_min = value;
        }
        if value > self.normal_max {
            self.normal_max = value;
        }
        
        // Медленное расширение диапазона
        if self.processed % 1000 == 0 {
            self.normal_min *= 0.995;
            self.normal_max *= 1.005;
        }
    }
    
    fn update_threshold(&mut self, score: f32, is_anomaly: bool) {
        self.recent_scores[self.recent_idx] = score;
        self.recent_anomalies[self.recent_idx] = is_anomaly;
        self.recent_idx = (self.recent_idx + 1) % 100;
        if self.recent_idx == 0 {
            self.recent_filled = true;
        }
        
        if self.recent_filled && self.processed % 50 == 0 {
            let len = 100;
            let anomaly_count = self.recent_anomalies.iter().filter(|&&x| x).count();
            let current_fpr = anomaly_count as f32 / len as f32;
            
            if current_fpr > TARGET_FPR {
                self.threshold = (self.threshold * 1.02).min(0.95);
            } else if current_fpr < TARGET_FPR * 0.3 {
                self.threshold = (self.threshold * 0.99).max(0.6);
            }
        }
    }
    
    pub fn get_threshold(&self) -> f32 {
        if self.training { 1.0 } else { self.threshold }
    }
    
    pub fn reset(&mut self) {
        *self = Self::new();
    }
}