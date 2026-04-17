//! Эксперт для обработки одного потока сигнала

use crate::rcf::RcfTree;
use crate::learning::AdaptiveThreshold;
use libm::powf;
use alloc::boxed::Box;

const NUM_TREES: usize = 15;
const INITIAL_THRESHOLD: f32 = 0.5;
const TARGET_FPR: f32 = 1.0;
const MAX_TREE_DEPTH: usize = 20;

pub struct SignalExpert {
    trees: Box<[RcfTree; NUM_TREES]>,
    threshold: AdaptiveThreshold,
    processed_count: usize,
    value_min: f32,
    value_max: f32,
}

impl SignalExpert {
    pub fn new() -> Self {
        let trees = Box::new([
            RcfTree::new(), RcfTree::new(), RcfTree::new(), RcfTree::new(), RcfTree::new(),
            RcfTree::new(), RcfTree::new(), RcfTree::new(), RcfTree::new(), RcfTree::new(),
            RcfTree::new(), RcfTree::new(), RcfTree::new(), RcfTree::new(), RcfTree::new(),
        ]);
        
        Self {
            trees,
            threshold: AdaptiveThreshold::new(INITIAL_THRESHOLD, TARGET_FPR),
            processed_count: 0,
            value_min: f32::MAX,
            value_max: f32::MIN,
        }
    }
    
    pub fn process(&mut self, value: f32) -> f32 {
        self.processed_count += 1;
        
        if value < self.value_min {
            self.value_min = value;
        }
        if value > self.value_max {
            self.value_max = value;
        }
        
        let score = self.compute_rcf_score(value);
        let is_anomaly = score > self.threshold.get_threshold();
        self.threshold.update(score, is_anomaly);
        
        score
    }
    
    fn compute_rcf_score(&mut self, value: f32) -> f32 {
        let mut total_depth = 0;
        
        for tree in self.trees.iter_mut() {
            let result = tree.insert_with_stats(value);
            total_depth += result.depth;
        }
        
        let avg_depth = total_depth as f32 / NUM_TREES as f32;
        
        // При достаточном обучении (100+ образцов):
        // Норма: глубина маленькая (2-5) → score НИЗКИЙ
        // Аномалия: глубина большая (8-12) → score ВЫСОКИЙ
        let depth_score = avg_depth / MAX_TREE_DEPTH as f32;
        
        // Степень 1.2 даёт хороший контраст
        let score = powf(depth_score, 1.5);
        
        if score < 0.0 {
            0.0
        } else if score > 1.0 {
            1.0
        } else {
            score
        }
    }
    
    pub fn process_with_depth(&mut self, value: f32) -> (f32, f32) {
        self.processed_count += 1;
        
        if value < self.value_min {
            self.value_min = value;
        }
        if value > self.value_max {
            self.value_max = value;
        }
        
        let mut total_depth = 0;
        
        for tree in self.trees.iter_mut() {
            let result = tree.insert_with_stats(value);
            total_depth += result.depth;
        }
        
        let avg_depth = total_depth as f32 / NUM_TREES as f32;
        
        let depth_score = avg_depth / MAX_TREE_DEPTH as f32;
        let score = powf(depth_score, 1.2);
        let score = if score < 0.0 { 0.0 } else if score > 1.0 { 1.0 } else { score };
        
        let is_anomaly = score > self.threshold.get_threshold();
        self.threshold.update(score, is_anomaly);
        
        (score, avg_depth)
    }
    
    pub fn get_threshold(&self) -> f32 {
        self.threshold.get_threshold()
    }
    
    pub fn reset(&mut self) {
        self.trees = Box::new([
            RcfTree::new(), RcfTree::new(), RcfTree::new(), RcfTree::new(), RcfTree::new(),
            RcfTree::new(), RcfTree::new(), RcfTree::new(), RcfTree::new(), RcfTree::new(),
            RcfTree::new(), RcfTree::new(), RcfTree::new(), RcfTree::new(), RcfTree::new(),
        ]);
        self.threshold = AdaptiveThreshold::new(INITIAL_THRESHOLD, TARGET_FPR);
        self.processed_count = 0;
        self.value_min = f32::MAX;
        self.value_max = f32::MIN;
    }
    
    pub fn processed_count(&self) -> usize {
        self.processed_count
    }
}