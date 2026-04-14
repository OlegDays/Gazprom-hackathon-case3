//! Дерево случайного разреза (Random Cut Tree)

use super::node::Node;
use alloc::boxed::Box;

const MAX_NODES: usize = 512;
const MIN_SPLIT_SAMPLES: u16 = 3;   // Уменьшаем до 5 для более частого разделения
const MAX_DEPTH: usize = 20;

pub struct InsertResult {
    pub depth: usize,
    pub leaf_samples: u16,
}

pub struct RcfTree {
    nodes: Box<[Node; MAX_NODES]>,
    node_count: usize,
}

impl RcfTree {
    pub fn new() -> Self {
        let mut nodes = Box::new([Node::new(); MAX_NODES]);
        nodes[0] = Node::new();
        Self {
            nodes,
            node_count: 1,
        }
    }
    
    pub fn insert_with_stats(&mut self, value: f32) -> InsertResult {
        let mut current = 0;
        let mut depth = 0;
        
        loop {
            self.nodes[current].update_bounds(value);
            
            // Проверяем, нужно ли разделить
            if self.nodes[current].is_leaf() && self.nodes[current].should_split(MIN_SPLIT_SAMPLES) {
                self.split_leaf(current);
            }
            
            // Если лист или достигли максимальной глубины
            if self.nodes[current].is_leaf() || depth >= MAX_DEPTH {
                return InsertResult {
                    depth,
                    leaf_samples: self.nodes[current].sample_count,
                };
            }
            
            // Идём влево или вправо
            if value <= self.nodes[current].split_value {
                if self.nodes[current].left == -1 {
                    self.create_leaf(current, true, value);
                    return InsertResult {
                        depth: depth + 1,
                        leaf_samples: 1,
                    };
                }
                current = self.nodes[current].left as usize;
            } else {
                if self.nodes[current].right == -1 {
                    self.create_leaf(current, false, value);
                    return InsertResult {
                        depth: depth + 1,
                        leaf_samples: 1,
                    };
                }
                current = self.nodes[current].right as usize;
            }
            depth += 1;
        }
    }
    
    pub fn insert(&mut self, value: f32) -> usize {
        self.insert_with_stats(value).depth
    }
    
    fn split_leaf(&mut self, node_idx: usize) {
        if self.node_count + 2 >= MAX_NODES {
            return;
        }
        
        self.nodes[node_idx].set_split_from_bounds();
        
        let left_idx = self.node_count;
        let right_idx = self.node_count + 1;
        self.node_count += 2;
        
        self.nodes[node_idx].left = left_idx as i32;
        self.nodes[node_idx].right = right_idx as i32;
        
        self.nodes[left_idx] = Node::new();
        self.nodes[right_idx] = Node::new();
    }
    
    fn create_leaf(&mut self, parent_idx: usize, is_left: bool, value: f32) {
        if self.node_count >= MAX_NODES {
            return;
        }
        
        let leaf_idx = self.node_count;
        self.node_count += 1;
        
        self.nodes[leaf_idx] = Node::new();
        self.nodes[leaf_idx].update_bounds(value);
        
        if is_left {
            self.nodes[parent_idx].left = leaf_idx as i32;
        } else {
            self.nodes[parent_idx].right = leaf_idx as i32;
        }
    }
}