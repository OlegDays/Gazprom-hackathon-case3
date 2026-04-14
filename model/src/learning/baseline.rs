//! Модуль адаптивного обучения порога

const HISTORY_BUFFER_SIZE: usize = 100;

pub struct AdaptiveThreshold {
    threshold: f32,
    target_fpr: f32,
    recent_scores: [f32; HISTORY_BUFFER_SIZE],
    recent_anomalies: [bool; HISTORY_BUFFER_SIZE],
    index: usize,
    filled: bool,
}

impl AdaptiveThreshold {
    pub fn new(_initial: f32, target_fpr: f32) -> Self {
        Self {
            threshold: 0.7,  // Фиксированный начальный порог
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
        
        // Корректируем порог каждые 20 образцов
        if self.filled && self.index % 20 == 0 {
            self.adjust_threshold();
        }
    }
    
    fn adjust_threshold(&mut self) {
        let mut anomaly_count = 0;
        for i in 0..HISTORY_BUFFER_SIZE {
            if self.recent_anomalies[i] {
                anomaly_count += 1;
            }
        }
        
        let current_fpr = anomaly_count as f32 / HISTORY_BUFFER_SIZE as f32;
        
        // Простая корректировка
        if current_fpr > self.target_fpr {
            self.threshold += 0.02;
        } else if current_fpr < self.target_fpr * 0.5 {
            self.threshold -= 0.01;
        }
        
        // Ограничиваем порог
        if self.threshold < 0.5 {
            self.threshold = 0.5;
        } else if self.threshold > 0.9 {
            self.threshold = 0.9;
        }
    }
    
    pub fn get_threshold(&self) -> f32 {
        self.threshold
    }
}