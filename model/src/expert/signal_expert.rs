use crate::rcf::tree::RcfTree;
use crate::learning::baseline::AdaptiveThreshold;
use crate::rcf::node::Node;
use crate::rcf::tree::{MAX_NODES_PER_TREE, WINDOW_SIZE};

pub const NUM_TREES: usize = 25;

pub struct SignalExpert {
    pub trees: [RcfTree; NUM_TREES],
    pub threshold: AdaptiveThreshold,
}

impl SignalExpert {
    pub const fn new(_base_node_offset: usize, _base_window_offset: usize, target_fpr: f32) -> Self {
        let placeholder_tree = RcfTree::new(0, 0, 0);
        Self {
            trees: [placeholder_tree; NUM_TREES],
            threshold: AdaptiveThreshold::new(0.30, target_fpr),
        }
    }

    pub fn init(&mut self, base_node_offset: usize, base_window_offset: usize) {
        for i in 0..NUM_TREES {
            let node_offset = base_node_offset + i * MAX_NODES_PER_TREE;
            let window_offset = base_window_offset + i * WINDOW_SIZE;
            let seed = (node_offset as u32) ^ 0x9E3779B9;
            self.trees[i] = RcfTree::new(node_offset, window_offset, seed);
        }
    }

    pub fn process(&mut self, value: f32, global_nodes: &mut [Node], global_windows: &mut [f32]) -> f32 {
        let mut total_codisp = 0.0;
        for tree in &mut self.trees {
            total_codisp += tree.insert(value, global_nodes, global_windows);
        }
        let raw_score = total_codisp / NUM_TREES as f32;
        // Нормализация: эмпирически raw_score для нормы ~0.1-0.3, для аномалий ~0.5-1.0
        let normalized = (raw_score * 3.0).min(1.0);
        
        let is_anomaly = normalized > self.threshold.get_threshold();
        self.threshold.update(normalized, is_anomaly);
        normalized
    }

    pub fn get_threshold(&self) -> f32 {
        self.threshold.get_threshold()
    }
}