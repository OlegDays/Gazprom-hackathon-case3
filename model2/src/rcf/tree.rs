//! Дерево случайного разреза

use super::node::Node;

const MAX_NODES: usize = 256;
const MIN_SPLIT_SAMPLES: u16 = 8;
const MAX_DEPTH: usize = 20;

pub struct RcfTree {
    nodes: [Node; MAX_NODES],
    node_count: usize,
    depth_count: [usize; MAX_DEPTH + 1],
}

impl RcfTree {
    pub fn new() -> Self {
        Self {
            nodes: [Node::new(); MAX_NODES],
            node_count: 1,
            depth_count: [0; MAX_DEPTH + 1],
        }
    }
    
    pub fn insert(&mut self, value: f32) -> usize {
        let mut current = 0usize;
        let mut depth = 0usize;
        
        // Спуск по дереву
        while depth < MAX_DEPTH {
            let node = &mut self.nodes[current];
            node.update(value);
            
            // Если лист и пора разделиться
            if node.is_leaf() && node.should_split(MIN_SPLIT_SAMPLES) {
                self.split_node(current);
            }
            
            // Если остались в листе
            if self.nodes[current].is_leaf() {
                break;
            }
            
            // Выбор направления
            let split_val = self.nodes[current].split_value;
            current = if value <= split_val {
                if self.nodes[current].left == -1 {
                    self.create_leaf(current, true, value);
                    return depth + 1;
                }
                self.nodes[current].left as usize
            } else {
                if self.nodes[current].right == -1 {
                    self.create_leaf(current, false, value);
                    return depth + 1;
                }
                self.nodes[current].right as usize
            };
            depth += 1;
        }
        
        depth
    }
    
    pub fn get_codisp(&self, value: f32) -> f32 {
        // Упрощенная версия - только глубина
        let mut current = 0usize;
        let mut depth = 0usize;
        
        while depth < MAX_DEPTH {
            if self.nodes[current].is_leaf() {
                break;
            }
            
            current = if value <= self.nodes[current].split_value {
                if self.nodes[current].left == -1 { break; }
                self.nodes[current].left as usize
            } else {
                if self.nodes[current].right == -1 { break; }
                self.nodes[current].right as usize
            };
            depth += 1;
        }
        
        (MAX_DEPTH - depth) as f32 / MAX_DEPTH as f32
    }
    
    fn split_node(&mut self, idx: usize) {
        if self.node_count + 2 >= MAX_NODES {
            return;
        }
        
        self.nodes[idx].set_random_split();
        
        let left_idx = self.node_count;
        let right_idx = self.node_count + 1;
        self.node_count += 2;
        
        self.nodes[idx].left = left_idx as i16;
        self.nodes[idx].right = right_idx as i16;
        
        self.nodes[left_idx] = Node::new();
        self.nodes[right_idx] = Node::new();
    }
    
    fn create_leaf(&mut self, parent: usize, is_left: bool, value: f32) {
        if self.node_count >= MAX_NODES {
            return;
        }
        
        let leaf_idx = self.node_count;
        self.node_count += 1;
        
        self.nodes[leaf_idx] = Node::new();
        self.nodes[leaf_idx].update(value);
        
        if is_left {
            self.nodes[parent].left = leaf_idx as i16;
        } else {
            self.nodes[parent].right = leaf_idx as i16;
        }
    }
}