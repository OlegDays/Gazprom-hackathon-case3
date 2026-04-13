// ============================================================
// node.rs - Узел RCF дерева
// ============================================================
//! Минимальная структура узла для Random Cut Forest
//! Без зависимостей, только core

use core::cmp::Ordering;

/// Узел бинарного дерева для RCF алгоритма
#[derive(Debug, Clone, Copy)]
pub struct Node {
    /// Индекс левого ребенка (-1 если отсутствует)
    pub left: i32,
    
    /// Индекс правого ребенка (-1 если отсутствует)
    pub right: i32,
    
    /// Значение разделения (порог)
    pub split_value: f32,
    
    /// Минимальное значение в поддереве
    pub min_val: f32,
    
    /// Максимальное значение в поддереве
    pub max_val: f32,
    
    /// Количество точек в поддереве
    pub sample_count: u16,
}

impl Node {
    /// Создает новый пустой узел
    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            left: -1,
            right: -1,
            split_value: 0.0,
            min_val: f32::MAX,
            max_val: f32::MIN,
            sample_count: 0,
        }
    }
    
    /// Проверяет, является ли узел листом
    #[inline(always)]
    pub fn is_leaf(&self) -> bool {
        self.left == -1 && self.right == -1
    }
    
    /// Обновляет границы узла новым значением
    #[inline(always)]
    pub fn update_bounds(&mut self, value: f32) {
        if value < self.min_val {
            self.min_val = value;
        }
        if value > self.max_val {
            self.max_val = value;
        }
        self.sample_count += 1;
    }
    
    /// Вычисляет диапазон значений в узле
    #[inline(always)]
    pub fn range(&self) -> f32 {
        self.max_val - self.min_val
    }
    
    /// Проверяет, нужно ли разделять узел
    #[inline(always)]
    pub fn should_split(&self, min_samples: u16, min_range: f32) -> bool {
        self.sample_count >= min_samples && self.range() > min_range
    }
    
    /// Устанавливает значение разделения (медиана диапазона)
    #[inline(always)]
    pub fn set_split_from_bounds(&mut self) {
        self.split_value = self.min_val + self.range() / 2.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_node_new() {
        let node = Node::new();
        assert_eq!(node.left, -1);
        assert_eq!(node.right, -1);
        assert_eq!(node.sample_count, 0);
        assert_eq!(node.min_val, f32::MAX);
        assert_eq!(node.max_val, f32::MIN);
        assert!(node.is_leaf());
    }
    
    #[test]
    fn test_update_bounds() {
        let mut node = Node::new();
        node.update_bounds(5.0);
        assert_eq!(node.min_val, 5.0);
        assert_eq!(node.max_val, 5.0);
        assert_eq!(node.sample_count, 1);
        
        node.update_bounds(3.0);
        assert_eq!(node.min_val, 3.0);
        assert_eq!(node.max_val, 5.0);
        assert_eq!(node.sample_count, 2);
        
        node.update_bounds(7.0);
        assert_eq!(node.min_val, 3.0);
        assert_eq!(node.max_val, 7.0);
        assert_eq!(node.sample_count, 3);
    }
    
    #[test]
    fn test_range() {
        let mut node = Node::new();
        node.update_bounds(10.0);
        node.update_bounds(20.0);
        assert_eq!(node.range(), 10.0);
        
        node.update_bounds(15.0);
        assert_eq!(node.range(), 10.0);
    }
    
    #[test]
    fn test_should_split() {
        let mut node = Node::new();
        assert!(!node.should_split(10, 0.01));
        
        for i in 0..15 {
            node.update_bounds(i as f32);
        }
        assert!(node.should_split(10, 0.01));
        assert!(!node.should_split(20, 0.01));
    }
}