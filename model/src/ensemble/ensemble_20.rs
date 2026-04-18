//! Ансамбль из 20 экспертов для обработки 20 потоков данных

use crate::expert::SignalExpert;

pub struct Ensemble20 {
	experts : [SignalExpert; 20],
}

impl Ensemble20 {
    pub fn new() -> Self {
        Self {
            experts: [
                SignalExpert::new(), SignalExpert::new(), SignalExpert::new(), SignalExpert::new(), SignalExpert::new(),
                SignalExpert::new(), SignalExpert::new(), SignalExpert::new(), SignalExpert::new(), SignalExpert::new(),
                SignalExpert::new(), SignalExpert::new(), SignalExpert::new(), SignalExpert::new(), SignalExpert::new(),
                SignalExpert::new(), SignalExpert::new(), SignalExpert::new(), SignalExpert::new(), SignalExpert::new(),
            ],
        }
    }
    
    pub fn process_frame(&mut self, values: &[f32; 20]) -> FrameResult {
        let mut scores = [0.0; 20];
        let mut anomalies = [false; 20];
        
        for i in 0..20 {
            scores[i] = self.experts[i].process(values[i]);
            anomalies[i] = scores[i] > self.experts[i].get_threshold();
        }
        
        FrameResult { scores, anomalies }
    }
    
    pub fn get_thresholds(&self) -> [f32; 20] {
        let mut thresholds = [0.0; 20];
        for i in 0..20 {
            thresholds[i] = self.experts[i].get_threshold();
        }
        thresholds
    }
    
    pub fn reset(&mut self) {
        for expert in self.experts.iter_mut() {
            expert.reset();
        }
    }
}

pub struct FrameResult {
    pub scores: [f32; 20],
    pub anomalies: [bool; 20],
}
