// ============================================================
// tree.rs - RCF Дерево
// ============================================================
//! Одно Random Cut Forest дерево с фиксированным массивом узлов

use super::node::Node;

/// Максимальное количество узлов в дереве
/// 256 узлов достаточно для глубины 8 при полном бинарном дереве
const MAX_NODES: usize = 256;

/// Минимальное количество точек в узле для разделения
const MIN_SPLIT_SAMPLES: u16 = 10;

/// Минимальный диапазон для разделения
const MIN_SPLIT_RANGE: f32 = 0.01;

/// Максимальная глубина дерева
const MAX_DEPTH: usize = 10;

/// RCF Дерево для обнаружения аномалий
pub struct RcfTree {
    /// Фиксированный массив узлов (без динамической памяти)
    nodes: [Node; MAX_NODES],
    
    /// Количество использованных узлов
    node_count: usize,
}

impl RcfTree {
    /// Создает новое пустое дерево
    #[inline(always)]
    pub fn new() -> Self {
        let mut nodes = [Node::new(); MAX_NODES];
        nodes[0] = Node::new();  // Инициализируем корневой узел
        
        Self {
            nodes,
            node_count: 1,
        }
    }
    
    /// Вставляет значение в дерево и возвращает глубину вставки
    /// 
    /// Глубина используется для вычисления аномальности:
    /// - Нормальные точки: глубокая вставка (большая глубина)
    /// - Аномальные точки: мелкая вставка (малая глубина)
    #[inline(always)]
    pub fn insert(&mut self, value: f32) -> usize {
        let mut current = 0;
        let mut depth = 0;
        
        loop {
            // Обновляем статистику текущего узла
            self.nodes[current].update_bounds(value);
            
            // Если достигли листа или максимальной глубины
            if self.nodes[current].is_leaf() || depth >= MAX_DEPTH {
                self.try_expand(current, value);
                return depth;
            }
            
            // Определяем направление движения
            if value <= self.nodes[current].split_value {
                // Идем в левое поддерево
                if self.nodes[current].left == -1 {
                    self.create_leaf(current, true, value);
                    return depth + 1;
                }
                current = self.nodes[current].left as usize;
            } else {
                // Идем в правое поддерево
                if self.nodes[current].right == -1 {
                    self.create_leaf(current, false, value);
                    return depth + 1;
                }
                current = self.nodes[current].right as usize;
            }
            depth += 1;
        }
    }
    
    /// Пытается расширить дерево, разделив лист
    #[inline(always)]
    fn try_expand(&mut self, node_idx: usize, value: f32) {
        if self.nodes[node_idx].should_split(MIN_SPLIT_SAMPLES, MIN_SPLIT_RANGE) {
            self.split_leaf(node_idx);
        }
    }
    
    /// Разделяет листовой узел на два дочерних
    #[inline(always)]
    fn split_leaf(&mut self, node_idx: usize) {
        if self.node_count + 2 >= MAX_NODES {
            return;  // Достигнут лимит узлов
        }
        
        // Устанавливаем значение разделения
        self.nodes[node_idx].set_split_from_bounds();
        
        // Создаем два дочерних узла
        let left_idx = self.node_count;
        let right_idx = self.node_count + 1;
        self.node_count += 2;
        
        self.nodes[node_idx].left = left_idx as i32;
        self.nodes[node_idx].right = right_idx as i32;
        
        // Инициализируем дочерние узлы
        self.nodes[left_idx] = Node::new();
        self.nodes[right_idx] = Node::new();
    }
    
    /// Создает новый листовой узел
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
    
    /// Сбрасывает дерево в начальное состояние
    #[inline(always)]
    pub fn reset(&mut self) {
        *self = Self::new();
    }
    
    /// Возвращает количество использованных узлов
    #[inline(always)]
    pub fn node_count(&self) -> usize {
        self.node_count
    }
}

impl Default for RcfTree {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_new_tree() {
        let tree = RcfTree::new();
        assert_eq!(tree.node_count(), 1);
    }
    
    #[test]
    fn test_insert_normal_values() {
        let mut tree = RcfTree::new();
        
        // Вставляем нормальные точки (близкие друг к другу)
        for i in 0..50 {
            let depth = tree.insert(10.0 + (i as f32) * 0.1);
            // Глубина должна увеличиваться с количеством точек
            if i > 20 {
                assert!(depth > 0);
            }
        }
    }
    
    #[test]
    fn test_insert_anomaly() {
        let mut tree = RcfTree::new();
        
        // Вставляем нормальные точки
        for i in 0..30 {
            tree.insert(10.0);
        }
        
        // Вставляем аномалию
        let anomaly_depth = tree.insert(100.0);
        
        // Аномалия должна иметь малую глубину (обычно 0-2)
        assert!(anomaly_depth <= 3);
    }
    
    #[test]
    fn test_reset() {
        let mut tree = RcfTree::new();
        
        for i in 0..50 {
            tree.insert(i as f32);
        }
        
        let old_count = tree.node_count();
        assert!(old_count > 1);
        
        tree.reset();
        assert_eq!(tree.node_count(), 1);
    }
}