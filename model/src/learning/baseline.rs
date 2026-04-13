// ============================================================
// baseline.rs - Online learning для адаптивной базовой линии
// ============================================================

use libm::sqrtf;

const MEAN_LEARNING_RATE: f32 = 0.1;
const STD_LEARNING_RATE: f32 = 0.05;
const MIN_SAMPLES_FOR_BASELINE: usize = 100;
const MAX_STD_DEV: f32 = 100.0;
const MIN_STD_DEV: f32 = 0.001;
const HISTORY_BUFFER_SIZE: usize = 100;

/// Адаптивная базовая линия для одного потока данных
pub struct AdaptiveBaseline {
    mean: f32,
    std_dev: f32,
    history: [f32; HISTORY_BUFFER_SIZE],
    history_pos: usize,
    history_count: usize,
    total_samples: usize,
}

impl AdaptiveBaseline {
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            mean: 0.0,
            std_dev: 1.0,
            history: [0.0; HISTORY_BUFFER_SIZE],
            history_pos: 0,
            history_count: 0,
            total_samples: 0,
        }
    }
    
    #[inline(always)]
    pub fn update(&mut self, value: f32) -> f32 {
        self.total_samples += 1;
        
        self.history[self.history_pos] = value;
        self.history_pos = (self.history_pos + 1) % HISTORY_BUFFER_SIZE;
        if self.history_count < HISTORY_BUFFER_SIZE {
            self.history_count += 1;
        }
        
        if self.history_count >= MIN_SAMPLES_FOR_BASELINE {
            if self.total_samples % 10 == 0 {
                self.full_recalc();
            } else {
                self.fast_update(value);
            }
        }
        
        self.normalize(value)
    }
    
    #[inline(always)]
    fn fast_update(&mut self, value: f32) {
        let delta = value - self.mean;
        self.mean += delta * MEAN_LEARNING_RATE;
        
        let abs_delta = delta.abs();
        let std_delta = abs_delta - self.std_dev;
        self.std_dev += std_delta * STD_LEARNING_RATE;
        
        self.std_dev = self.std_dev.clamp(MIN_STD_DEV, MAX_STD_DEV);
    }
    
    #[inline(always)]
    fn full_recalc(&mut self) {
        if self.history_count == 0 {
            return;
        }
        
        let mut sum = 0.0;
        for i in 0..self.history_count {
            sum += self.history[i];
        }
        let new_mean = sum / self.history_count as f32;
        
        let mut sum_sq = 0.0;
        for i in 0..self.history_count {
            let diff = self.history[i] - new_mean;
            sum_sq += diff * diff;
        }
        let new_std = sqrtf(sum_sq / self.history_count as f32);
        
        self.mean = self.mean * 0.7 + new_mean * 0.3;
        self.std_dev = self.std_dev * 0.7 + new_std * 0.3;
        self.std_dev = self.std_dev.clamp(MIN_STD_DEV, MAX_STD_DEV);
    }
    
    #[inline(always)]
    pub fn normalize(&self, value: f32) -> f32 {
        (value - self.mean) / self.std_dev
    }
    
    #[inline(always)]
    pub fn get_mean(&self) -> f32 {
        self.mean
    }
    
    #[inline(always)]
    pub fn get_std_dev(&self) -> f32 {
        self.std_dev
    }
    
    #[inline(always)]
    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

impl Default for AdaptiveBaseline {
    fn default() -> Self {
        Self::new()
    }
}

/// Адаптивный порог аномальности
pub struct AdaptiveThreshold {
    threshold: f32,
    target_false_positive_rate: f32,
    adaptation_rate: f32,
    anomaly_count: usize,
    total_count: usize,
}

impl AdaptiveThreshold {
    #[inline(always)]
    pub fn new(initial_threshold: f32, target_fpr: f32) -> Self {
        Self {
            threshold: initial_threshold,
            target_false_positive_rate: target_fpr,
            adaptation_rate: 0.001,
            anomaly_count: 0,
            total_count: 0,
        }
    }
    
    #[inline(always)]
    pub fn update(&mut self, _score: f32, is_anomaly: bool) -> f32 {
        self.total_count += 1;
        if is_anomaly {
            self.anomaly_count += 1;
        }
        
        if self.total_count >= 1000 {
            let actual_fpr = self.anomaly_count as f32 / self.total_count as f32;
            
            if actual_fpr > self.target_false_positive_rate {
                self.threshold += self.adaptation_rate;
            } else if actual_fpr < self.target_false_positive_rate * 0.5 {
                self.threshold -= self.adaptation_rate;
            }
            
            self.threshold = self.threshold.clamp(0.5, 0.95);
            self.anomaly_count = 0;
            self.total_count = 0;
        }
        
        self.threshold
    }
    
    #[inline(always)]
    pub fn get_threshold(&self) -> f32 {
        self.threshold
    }
    
    #[inline(always)]
    pub fn set_threshold(&mut self, threshold: f32) {
        self.threshold = threshold.clamp(0.1, 0.99);
    }
    
    #[inline(always)]
    pub fn reset(&mut self, initial_threshold: f32) {
        self.threshold = initial_threshold;
        self.anomaly_count = 0;
        self.total_count = 0;
    }
}