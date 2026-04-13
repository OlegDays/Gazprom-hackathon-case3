// ============================================================
// ensemble_20.rs - Ансамбль из 20 независимых экспертов
// ============================================================

use crate::expert::SignalExpert;

pub const NUM_EXPERTS: usize = 20;

/// Ансамбль из 20 экспертов
pub struct Ensemble20 {
    experts: Box<[Box<SignalExpert>; NUM_EXPERTS]>,
}

impl Ensemble20 {
    #[inline(always)]
    pub fn new() -> Self {
        let experts = Box::new([
            Box::new(SignalExpert::new()),
            Box::new(SignalExpert::new()),
            Box::new(SignalExpert::new()),
            Box::new(SignalExpert::new()),
            Box::new(SignalExpert::new()),
            Box::new(SignalExpert::new()),
            Box::new(SignalExpert::new()),
            Box::new(SignalExpert::new()),
            Box::new(SignalExpert::new()),
            Box::new(SignalExpert::new()),
            Box::new(SignalExpert::new()),
            Box::new(SignalExpert::new()),
            Box::new(SignalExpert::new()),
            Box::new(SignalExpert::new()),
            Box::new(SignalExpert::new()),
            Box::new(SignalExpert::new()),
            Box::new(SignalExpert::new()),
            Box::new(SignalExpert::new()),
            Box::new(SignalExpert::new()),
            Box::new(SignalExpert::new()),
        ]);
        
        Self { experts }
    }
    
    #[inline(always)]
    pub fn process_batch(&mut self, data: [f32; NUM_EXPERTS]) -> [f32; NUM_EXPERTS] {
        let mut results = [0.0; NUM_EXPERTS];
        
        for i in 0..NUM_EXPERTS {
            results[i] = self.experts[i].process(data[i]);
        }
        
        results
    }
    
    #[inline(always)]
    pub fn process_batch_binary(&mut self, data: [f32; NUM_EXPERTS]) -> [bool; NUM_EXPERTS] {
        let mut results = [false; NUM_EXPERTS];
        
        for i in 0..NUM_EXPERTS {
            let score = self.experts[i].process(data[i]);
            results[i] = self.experts[i].is_anomaly(score);
        }
        
        results
    }
    
    #[inline(always)]
    pub fn reset_all(&mut self) {
        for expert in self.experts.iter_mut() {
            expert.reset();
        }
    }
}

impl Default for Ensemble20 {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}