// ============================================================
// tree.rs - RCF Дерево
// ============================================================

use super::node::Node;

const MAX_NODES: usize = 256;
const MIN_SPLIT_SAMPLES: u16 = 10;
const MAX_DEPTH: usize = 10;

pub struct RcfTree {
    nodes: [Node; MAX_NODES],
    node_count: usize,
}

impl RcfTree {
    #[inline(always)]
    pub fn new() -> Self {
        let mut nodes = [Node::new(); MAX_NODES];
        nodes[0] = Node::new();
        
        Self {
            nodes,
            node_count: 1,
        }
    }
    
    #[inline(always)]
    pub fn insert(&mut self, value: f32) -> usize {
        let mut current = 0;
        let mut depth = 0;
        
        loop {
            self.nodes[current].update_bounds(value);
            
            // Если лист и пора разделить
            if self.nodes[current].is_leaf() && self.nodes[current].should_split(MIN_SPLIT_SAMPLES) {
                self.split_leaf(current);
            }
            
            // Если все еще лист или достигли глубины
            if self.nodes[current].is_leaf() || depth >= MAX_DEPTH {
                return depth;
            }
            
            // Идем вниз
            if value <= self.nodes[current].split_value {
                if self.nodes[current].left == -1 {
                    self.create_leaf(current, true, value);
                    return depth + 1;
                }
                current = self.nodes[current].left as usize;
            } else {
                if self.nodes[current].right == -1 {
                    self.create_leaf(current, false, value);
                    return depth + 1;
                }
                current = self.nodes[current].right as usize;
            }
            depth += 1;
        }
    }
    
    #[inline(always)]
    fn split_leaf(&mut self, node_idx: usize) {
        if self.node_count + 2 >= MAX_NODES {
            return;
        }
        
        // Устанавливаем значение разделения
        self.nodes[node_idx].set_split_from_bounds();
        
        let left_idx = self.node_count;
        let right_idx = self.node_count + 1;
        self.node_count += 2;
        
        self.nodes[node_idx].left = left_idx as i32;
        self.nodes[node_idx].right = right_idx as i32;
        
        self.nodes[left_idx] = Node::new();
        self.nodes[right_idx] = Node::new();
        
        // Распределяем существующие точки? Нет, RCF не требует ребалансировки
    }
    
    #[inline(always)]
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
    
    #[inline(always)]
    pub fn reset(&mut self) {
        *self = Self::new();
    }
    
    #[inline(always)]
    pub fn node_count(&self) -> usize {
        self.node_count
    }
}

impl Default for RcfTree {
    fn default() -> Self {
        Self::new()
    }
}