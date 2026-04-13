// ============================================================
// signal_expert.rs
// ============================================================

use crate::rcf::RcfTree;
use crate::learning::{AdaptiveBaseline, AdaptiveThreshold};

const NUM_TREES: usize = 10;
const INITIAL_THRESHOLD: f32 = 0.4;
const TARGET_FPR: f32 = 0.05;
const MAX_TREE_DEPTH: usize = 8;

pub struct SignalExpert {
    trees: [RcfTree; NUM_TREES],
    baseline: AdaptiveBaseline,
    threshold: AdaptiveThreshold,
    processed_count: usize,
}

impl SignalExpert {
    #[inline(always)]
    pub fn new() -> Self {
        let trees = [
            RcfTree::new(), RcfTree::new(), RcfTree::new(), RcfTree::new(), RcfTree::new(),
            RcfTree::new(), RcfTree::new(), RcfTree::new(), RcfTree::new(), RcfTree::new(),
        ];
        
        Self {
            trees,
            baseline: AdaptiveBaseline::new(),
            threshold: AdaptiveThreshold::new(INITIAL_THRESHOLD, TARGET_FPR),
            processed_count: 0,
        }
    }
    
    #[inline(always)]
    pub fn process(&mut self, value: f32) -> f32 {
        self.processed_count += 1;
        
        let normalized = self.baseline.update(value);
        let score = self.compute_rcf_score(normalized);
        
        let is_anomaly = score > self.threshold.get_threshold();
        self.threshold.update(score, is_anomaly);
        
        score
    }
    
    #[inline(always)]
    fn compute_rcf_score(&mut self, value: f32) -> f32 {
        let mut total_depth = 0;
        
        for tree in &mut self.trees {
            total_depth += tree.insert(value);
        }
        
        let avg_depth = total_depth as f32 / NUM_TREES as f32;
        
        // Простая линейная формула
        let score = 1.0 - (avg_depth / MAX_TREE_DEPTH as f32);
        
        // Раздуваем умножением
        let boosted = (score * 1.5).min(1.0);
        
        boosted.clamp(0.0, 1.0)
    }
    
    #[inline(always)]
    pub fn is_anomaly(&self, score: f32) -> bool {
        score > self.threshold.get_threshold()
    }
    
    #[inline(always)]
    pub fn get_threshold(&self) -> f32 {
        self.threshold.get_threshold()
    }
    
    #[inline(always)]
    pub fn reset(&mut self) {
        *self = Self::new();
    }
    
    #[inline(always)]
    pub fn processed_count(&self) -> usize {
        self.processed_count
    }
}

impl Default for SignalExpert {
    fn default() -> Self {
        Self::new()
    }
}