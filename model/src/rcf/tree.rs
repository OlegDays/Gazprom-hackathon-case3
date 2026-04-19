// реализвция дерева

use crate::rcf::node::Node;

// основные характеристики дерева
pub const MAX_NODES_PER_TREE: usize = 200;
pub const MIN_SPLIT_SAMPLES: u16 = 12;
pub const MAX_DEPTH: usize = 15;
pub const WINDOW_SIZE: usize = 900;

fn variance(node: &Node) -> f32 {
    if node.count <= 1 { 0.0 } else {
        let mean = node.sum / node.count as f32;
        (node.sum_sq / node.count as f32) - (mean * mean)
    }
}

fn update_stats(node: &mut Node, value: f32) {
    node.sum += value;
    node.sum_sq += value * value;
    node.count += 1;
    if value < node.min_val { node.min_val = value; }
    if value > node.max_val { node.max_val = value; }
}

fn remove_stats(node: &mut Node, value: f32) {
    if node.count > 0 {
        node.sum -= value;
        node.sum_sq -= value * value;
        node.count -= 1;
    }
    if node.count == 0 {
        node.min_val = f32::MAX;
        node.max_val = f32::MIN;
    }
}

fn is_leaf(node: &Node) -> bool { !node.is_split }

// проверка на надобность разделения
fn should_split(node: &Node, depth: usize) -> bool {
    !node.is_split
        && node.count >= MIN_SPLIT_SAMPLES
        && (node.max_val - node.min_val) > 1e-6
        && depth < MAX_DEPTH
}

fn fastrand_f32(seed: &mut u32) -> f32 {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 17;
    *seed ^= *seed << 5;
    (*seed as f32) / 4294967296.0
}
// разделение
fn set_random_split(node: &mut Node, rng_seed: &mut u32) {
    let r = node.min_val + (node.max_val - node.min_val) * fastrand_f32(rng_seed);
    node.split_value = r;
    node.is_split = true;
}

#[derive(Copy, Clone)]
pub struct RcfTree {
    pub node_offset: usize,
    pub window_offset: usize,
    pub node_count: usize,
    pub window_pos: usize,
    pub window_filled: bool,
    pub rng_seed: u32,
}
// само дерево
impl RcfTree {
    pub const fn new(node_offset: usize, window_offset: usize, rng_seed: u32) -> Self {
        Self {
            node_offset,
            window_offset,
            node_count: 1,
            window_pos: 0,
            window_filled: false,
            rng_seed,
        }
    }

    fn nodes<'a>(&self, global_nodes: &'a mut [Node]) -> &'a mut [Node] {
        &mut global_nodes[self.node_offset..self.node_offset + MAX_NODES_PER_TREE]
    }
    // создание окна адаптации
    fn window<'a>(&self, global_windows: &'a mut [f32]) -> &'a mut [f32] {
        &mut global_windows[self.window_offset..self.window_offset + WINDOW_SIZE]
    }
    // вставка значений
    pub fn insert(&mut self, value: f32, global_nodes: &mut [Node], global_windows: &mut [f32]) -> f32 {
        let nodes = self.nodes(global_nodes);
        let window = self.window(global_windows);

        if self.window_filled {
            let old_val = window[self.window_pos];
            self.delete_point(old_val, nodes);
        }

        let codisp = self.insert_point(value, nodes);

        window[self.window_pos] = value;
        self.window_pos = (self.window_pos + 1) % WINDOW_SIZE;
        if self.window_pos == 0 {
            self.window_filled = true;
        }
        codisp
    }

    fn insert_point(&mut self, value: f32, nodes: &mut [Node]) -> f32 {
        let mut current = 0;
        let mut depth = 0;
        let mut total_var_change = 0.0;
        let mut visited = 0;

        loop {
            let old_var = variance(&nodes[current]);
            update_stats(&mut nodes[current], value);
            let new_var = variance(&nodes[current]);
            // Относительное изменение дисперсии (ключевое улучшение)
            let var_change = if old_var > 1e-6 {
                (new_var - old_var).abs() / old_var
            } else {
                (new_var - old_var).abs() * 10.0
            };
            total_var_change += var_change;
            visited += 1;

            if should_split(&nodes[current], depth) {
                self.split_leaf(current, nodes);
            }

            if is_leaf(&nodes[current]) || depth >= MAX_DEPTH {
                break;
            }

            let next = if value <= nodes[current].split_value {
                nodes[current].left
            } else {
                nodes[current].right
            };

            if next == -1 {
                self.create_leaf(current, value <= nodes[current].split_value, value, nodes);
                break;
            }
            current = next as usize;
            depth += 1;
        }

        if visited > 0 { total_var_change / visited as f32 } else { 0.0 }
    }

    fn delete_point(&mut self, value: f32, nodes: &mut [Node]) {
        let mut current = 0;
        let mut depth = 0;
        loop {
            remove_stats(&mut nodes[current], value);
            if is_leaf(&nodes[current]) || depth >= MAX_DEPTH {
                break;
            }
            let next = if value <= nodes[current].split_value {
                nodes[current].left
            } else {
                nodes[current].right
            };
            if next == -1 {
                break;
            }
            current = next as usize;
            depth += 1;
        }
    }

    fn split_leaf(&mut self, node_idx: usize, nodes: &mut [Node]) {
        if self.node_count + 2 >= MAX_NODES_PER_TREE {
            return;
        }
        let left_idx = self.node_count;
        let right_idx = self.node_count + 1;
        self.node_count += 2;

        set_random_split(&mut nodes[node_idx], &mut self.rng_seed);
        nodes[node_idx].left = left_idx as i32;
        nodes[node_idx].right = right_idx as i32;

        nodes[left_idx] = Node::new();
        nodes[right_idx] = Node::new();
    }

    fn create_leaf(&mut self, parent_idx: usize, is_left: bool, value: f32, nodes: &mut [Node]) {
        if self.node_count >= MAX_NODES_PER_TREE {
            return;
        }
        let leaf_idx = self.node_count;
        self.node_count += 1;
        nodes[leaf_idx] = Node::new();
        update_stats(&mut nodes[leaf_idx], value);
        if is_left {
            nodes[parent_idx].left = leaf_idx as i32;
        } else {
            nodes[parent_idx].right = leaf_idx as i32;
        }
    }
}