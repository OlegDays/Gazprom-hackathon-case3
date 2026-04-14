//! Узел дерева случайного разреза (Random Cut Forest)

#[derive(Debug, Clone, Copy)]
pub struct Node {
    pub left: i32,
    pub right: i32,
    pub split_value: f32,
    pub min_val: f32,
    pub max_val: f32,
    pub sample_count: u16,
    pub split_count: u16,
}

impl Node {
    pub const fn new() -> Self {
        Self {
            left: -1,
            right: -1,
            split_value: 0.0,
            min_val: f32::MAX,
            max_val: f32::MIN,
            sample_count: 0,
            split_count: 0,
        }
    }
    
    pub fn is_leaf(&self) -> bool {
        self.left == -1 && self.right == -1
    }
    
    pub fn update_bounds(&mut self, value: f32) {
        if value < self.min_val {
            self.min_val = value;
        }
        if value > self.max_val {
            self.max_val = value;
        }
        self.sample_count += 1;
    }
    
    pub fn range(&self) -> f32 {
        self.max_val - self.min_val
    }
    
   pub fn should_split(&self, min_samples: u16) -> bool {
    // Разделяем если есть достаточно образцов И есть разброс
    self.sample_count >= min_samples 
        && self.split_count == 0 
        && self.range() > 0.01  // Требуем минимальный разброс
}
    
    pub fn set_split_from_bounds(&mut self) {
        if self.range() > 0.001 {
            // Разделяем посередине
            self.split_value = self.min_val + self.range() / 2.0;
        } else {
            // Для одинаковых значений - добавляем небольшое смещение
            self.split_value = self.min_val + 0.1;
        }
        self.split_count = 1;
    }
}