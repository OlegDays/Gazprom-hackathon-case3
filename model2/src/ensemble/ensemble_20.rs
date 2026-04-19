//! Ансамбль из 20 экспертов. Версия 2.2: Добавлено подавление предупреждения.
#![allow(dead_code)]

use crate::expert::SignalExpert;

pub const ENSEMBLE_SIZE: usize = 20;
const INITIAL_THRESHOLD: f32 = 0.45;

#[derive(Debug, Clone, Copy)]
pub struct FrameResult {
    pub scores: [f32; ENSEMBLE_SIZE],
    pub anomalies: [bool; ENSEMBLE_SIZE],
    pub anomaly_count: usize,
}

impl FrameResult {
    pub const fn new() -> Self {
        Self {
            scores: [0.0; ENSEMBLE_SIZE],
            anomalies: [false; ENSEMBLE_SIZE],
            anomaly_count: 0,
        }
    }
    
    #[inline]
    pub fn has_anomaly(&self) -> bool {
        self.anomaly_count > 0
    }
}

#[derive(Debug, Clone, Copy)]
pub struct EnsembleStats {
    pub total_frames: usize,
    pub anomaly_frames: usize,
    pub total_anomalies: usize,
    pub thresholds: [f32; ENSEMBLE_SIZE],
    pub avg_scores: [f32; ENSEMBLE_SIZE],
}

pub struct Ensemble20 {
    experts: [SignalExpert; ENSEMBLE_SIZE],
    stats: EnsembleStats,
    frame_counter: usize,
    initialized: bool,
    init_buffer: [[f32; ENSEMBLE_SIZE]; 400],
    init_count: usize,
}

impl Ensemble20 {
    pub const fn new() -> Self {
        Self {
            experts: [
                SignalExpert::new(), SignalExpert::new(), SignalExpert::new(), SignalExpert::new(),
                SignalExpert::new(), SignalExpert::new(), SignalExpert::new(), SignalExpert::new(),
                SignalExpert::new(), SignalExpert::new(), SignalExpert::new(), SignalExpert::new(),
                SignalExpert::new(), SignalExpert::new(), SignalExpert::new(), SignalExpert::new(),
                SignalExpert::new(), SignalExpert::new(), SignalExpert::new(), SignalExpert::new(),
            ],
            stats: EnsembleStats {
                total_frames: 0, anomaly_frames: 0, total_anomalies: 0,
                thresholds: [INITIAL_THRESHOLD; ENSEMBLE_SIZE], avg_scores: [0.0; ENSEMBLE_SIZE],
            },
            frame_counter: 0, initialized: false,
            init_buffer: [[0.0; ENSEMBLE_SIZE]; 400], init_count: 0,
        }
    }

    pub fn process_frame(&mut self, values: &[f32; ENSEMBLE_SIZE]) -> FrameResult {
        if !self.initialized {
            self.init_buffer[self.init_count] = *values;
            self.init_count += 1;
            if self.init_count >= 400 {
                for i in 0..self.init_count {
                    for ch in 0..ENSEMBLE_SIZE {
                        self.experts[ch].process_learning(self.init_buffer[i][ch]);
                    }
                }
                self.initialized = true;
            }
            return FrameResult::new();
        }

        self.frame_counter += 1;
        let mut result = FrameResult::new();

        for i in 0..ENSEMBLE_SIZE {
            result.scores[i] = self.experts[i].process(values[i]);
            self.stats.thresholds[i] = self.experts[i].get_threshold();
            result.anomalies[i] = result.scores[i] > self.stats.thresholds[i];
            
            if result.anomalies[i] {
                result.anomaly_count += 1;
                self.stats.total_anomalies += 1;
            }
            self.stats.avg_scores[i] = self.stats.avg_scores[i] * 0.99 + result.scores[i] * 0.01;
        }

        self.stats.total_frames += 1;
        if result.has_anomaly() { self.stats.anomaly_frames += 1; }
        
        if self.frame_counter % 5000 == 0 { self.adaptive_reset(); }
        
        result
    }

    fn adaptive_reset(&mut self) {
        let mut scores_with_indices: [(f32, usize); ENSEMBLE_SIZE] =
            core::array::from_fn(|i| (self.stats.avg_scores[i], i));
        
        for i in 0..ENSEMBLE_SIZE {
            for j in 0..ENSEMBLE_SIZE - i - 1 {
                if scores_with_indices[j].0 > scores_with_indices[j + 1].0 {
                    scores_with_indices.swap(j, j + 1);
                }
            }
        }
        let reset_count = ENSEMBLE_SIZE / 4;
        for i in 0..reset_count {
            let idx = scores_with_indices[i].1;
            self.experts[idx].reset();
            self.stats.avg_scores[idx] = 0.0;
            self.stats.thresholds[idx] = self.experts[idx].get_threshold();
        }
    }

    pub fn reset(&mut self) { *self = Self::new(); }
    
    pub fn get_stats(&self) -> EnsembleStats { self.stats }
}

impl Default for Ensemble20 {
    fn default() -> Self { Self::new() }
}

static mut G_ENSEMBLE: Ensemble20 = Ensemble20::new();

#[allow(static_mut_refs)]
pub fn get_instance() -> &'static mut Ensemble20 {
    unsafe { &mut G_ENSEMBLE }
}
