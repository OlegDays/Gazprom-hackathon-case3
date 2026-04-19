#[derive(Debug, Clone, Copy)]

// реализация листа
pub struct Node {
    pub left: i32,
    pub right: i32,
    pub split_value: f32,
    pub min_val: f32,
    pub max_val: f32,
    pub sum: f32,
    pub sum_sq: f32,
    pub count: u16,
    pub is_split: bool,
}

impl Node {
    pub const fn new() -> Self {
        Self {
            left: -1,
            right: -1,
            split_value: 0.0,
            min_val: f32::MAX,
            max_val: f32::MIN,
            sum: 0.0,
            sum_sq: 0.0,
            count: 0,
            is_split: false,
        }
    }
}