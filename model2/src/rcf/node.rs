//! Узел дерева случайного разреза

static mut RNG_STATE: u32 = 12345;

#[inline]
pub fn get_random() -> f32 {
    unsafe {
        RNG_STATE = RNG_STATE.wrapping_mul(1103515245).wrapping_add(12345);
        (RNG_STATE as f32) / (u32::MAX as f32)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Node {
    pub left: i16,
    pub right: i16,
    pub split_value: f32,
    pub min_val: f32,
    pub max_val: f32,
    pub sample_count: u16,
    pub split_count: u8,
    pub initialized: bool,
}

impl Node {
    pub const fn new() -> Self {
        Self {
            left: -1,
            right: -1,
            split_value: 0.0,
            min_val: 0.0,
            max_val: 0.0,
            sample_count: 0,
            split_count: 0,
            initialized: false,
        }
    }
    
    #[inline]
    pub fn update(&mut self, value: f32) {
        if !self.initialized {
            self.min_val = value;
            self.max_val = value;
            self.initialized = true;
        } else {
            if value < self.min_val { self.min_val = value; }
            if value > self.max_val { self.max_val = value; }
        }
        self.sample_count = self.sample_count.saturating_add(1);
    }
    
    #[inline]
    pub fn range(&self) -> f32 {
        if !self.initialized { 0.0 } else { self.max_val - self.min_val }
    }
    
    #[inline]
    pub fn is_leaf(&self) -> bool {
        self.left == -1 && self.right == -1
    }
    
    #[inline]
    pub fn should_split(&self, min_samples: u16) -> bool {
        self.sample_count >= min_samples && self.split_count == 0 && self.range() > 0.01
    }
    
    pub fn set_random_split(&mut self) {
        if !self.initialized || self.range() <= 0.001 {
            self.split_value = self.min_val + 0.1;
        } else {
            self.split_value = self.min_val + self.range() * get_random();
        }
        self.split_count = 1;
    }
}