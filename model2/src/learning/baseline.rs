//! Адаптивный порог

pub struct AdaptiveThreshold {
    threshold: f32,
    target_fpr: f32,
    history: [f32; 100],
    anomalies: [bool; 100],
    idx: usize,
    filled: bool,
}

impl AdaptiveThreshold {
    pub fn new(initial: f32, target_fpr_percent: f32) -> Self {
        Self {
            threshold: initial,
            target_fpr: target_fpr_percent / 100.0,
            history: [0.0; 100],
            anomalies: [false; 100],
            idx: 0,
            filled: false,
        }
    }
    
    pub fn update(&mut self, score: f32, is_anomaly: bool) {
        self.history[self.idx] = score;
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
        let len = if self.filled { 100 } else { self.idx };
        if len == 0 { return; }
        
        let anomaly_count = self.anomalies.iter().take(len).filter(|&&x| x).count();
        let current_fpr = anomaly_count as f32 / len as f32;
        
        if current_fpr > self.target_fpr {
            self.threshold += 0.02;
        } else if current_fpr < self.target_fpr * 0.3 {
            self.threshold -= 0.01;
        }
        
        self.threshold = self.threshold.clamp(0.45, 0.85);
    }
    
    pub fn get(&self) -> f32 {
        self.threshold
    }
}