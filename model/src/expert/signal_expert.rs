//! Эксперт для обработки одного потока сигнала

use crate::rcf::RcfTree;
use crate::learning::AdaptiveThreshold;
use libm::powf;
use alloc::boxed::Box;

const NUM_TREES: usize = 5;
const INITIAL_THRESHOLD: f32 = 0.75;
const TARGET_FPR: f32 = 5.0;
const MAX_TREE_DEPTH: usize = 20;
const RESET_INTERVAL: usize = 2000;
// Фильтр длительности
const MIN_ANOMALY_DURATION: usize = 2;

pub struct SignalExpert {
    trees: Box<[RcfTree; NUM_TREES]>,
    threshold: AdaptiveThreshold,
    processed_count: usize,
    anomaly_streak: usize,
}

impl SignalExpert {
    pub fn new() -> Self {
        let trees = Box::new([
            RcfTree::new(), RcfTree::new(), RcfTree::new(), RcfTree::new(), RcfTree::new(),
        ]);
        
        Self {
            trees,
            threshold: AdaptiveThreshold::new(INITIAL_THRESHOLD, TARGET_FPR),
            processed_count: 0,
            anomaly_streak: 0,
        }
    }
    
    pub fn process(&mut self, value: f32) -> f32 {
        self.processed_count += 1;
        
        if self.processed_count % RESET_INTERVAL == 0 {
            for i in 0..NUM_TREES / 3 {
                self.trees[i] = RcfTree::new();
            }
        }
        
        let raw_score = self.compute_raw_score(value);
        let is_raw_anomaly = raw_score > self.threshold.get_threshold();
        
        if is_raw_anomaly {
            self.anomaly_streak += 1;
        } else {
            self.anomaly_streak = 0;
        }
        
        let confirmed_anomaly = self.anomaly_streak >= MIN_ANOMALY_DURATION;
        self.threshold.update(raw_score, is_raw_anomaly);
        
        if confirmed_anomaly {
            1.0
        } else {
            raw_score * 0.5
        }
    }
    
   fn compute_raw_score(&mut self, value: f32) -> f32 {
    let mut total_depth = 0;
    
    for tree in self.trees.iter_mut() {
        let result = tree.insert_with_stats(value);
        total_depth += result.depth;
    }
    
    let avg_depth = total_depth as f32 / NUM_TREES as f32;
    
    let score = if avg_depth < 5.0 {
        0.85 + (5.0 - avg_depth) / 25.0
    } else if avg_depth < 8.0 {
        0.6 + (8.0 - avg_depth) / 10.0
    } else if avg_depth < 12.0 {
        0.2 + (12.0 - avg_depth) / 40.0
    } else {
        0.0
    };
    
    score.clamp(0.0, 1.0)
}
    
    pub fn get_threshold(&self) -> f32 {
        self.threshold.get_threshold()
    }
    
    pub fn reset(&mut self) {
        self.trees = Box::new([
            RcfTree::new(), RcfTree::new(), RcfTree::new(), RcfTree::new(), RcfTree::new(),
        ]);
        self.threshold = AdaptiveThreshold::new(INITIAL_THRESHOLD, TARGET_FPR);
        self.processed_count = 0;
        self.anomaly_streak = 0;
    }
    
    pub fn processed_count(&self) -> usize {
        self.processed_count
    }
}