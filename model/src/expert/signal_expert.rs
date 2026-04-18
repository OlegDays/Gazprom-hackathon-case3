//! Эксперт для обработки одного потока сигнала

use crate::rcf::RcfTree;
use crate::learning::AdaptiveThreshold;
use libm::powf;
use alloc::boxed::Box;

const NUM_TREES: usize = 25;
const INITIAL_THRESHOLD: f32 = 0.7;
const TARGET_FPR: f32 = 5.0;
const MAX_TREE_DEPTH: usize = 20;
const RESET_INTERVAL: usize = 2000;

pub struct SignalExpert {
    trees: Box<[RcfTree; NUM_TREES]>,
    threshold: AdaptiveThreshold,
    processed_count: usize,
    
    // История сырых значений для калибровки
    value_history: [f32; 200],
    history_idx: usize,
    history_filled: bool,
}

impl SignalExpert {
    pub fn new() -> Self {
        let trees = Box::new([
            RcfTree::new(), RcfTree::new(), RcfTree::new(), RcfTree::new(), RcfTree::new(),
            RcfTree::new(), RcfTree::new(), RcfTree::new(), RcfTree::new(), RcfTree::new(),
            RcfTree::new(), RcfTree::new(), RcfTree::new(), RcfTree::new(), RcfTree::new(),
            RcfTree::new(), RcfTree::new(), RcfTree::new(), RcfTree::new(), RcfTree::new(),
            RcfTree::new(), RcfTree::new(), RcfTree::new(), RcfTree::new(), RcfTree::new(),
        ]);
        
        Self {
            trees,
            threshold: AdaptiveThreshold::new(INITIAL_THRESHOLD, TARGET_FPR),
            processed_count: 0,
            value_history: [0.0; 200],
            history_idx: 0,
            history_filled: false,
        }
    }
    
    pub fn process(&mut self, value: f32) -> f32 {
        self.processed_count += 1;
        
        // Сохраняем значение в историю
        self.value_history[self.history_idx] = value;
        self.history_idx = (self.history_idx + 1) % 200;
        if self.history_idx == 0 {
            self.history_filled = true;
        }
        
        // Периодический частичный сброс
        if self.processed_count % RESET_INTERVAL == 0 {
            self.partial_reset();
        }
        
        // 1. Вычисляем скор RCF
        let score = self.compute_rcf_score(value);
        
        // 2. Корректируем скор на основе локальной волатильности
        let adjusted_score = if self.history_filled {
            self.adjust_score_by_volatility(score, value)
        } else {
            score
        };
        
        // 3. Применяем порог
        let is_anomaly = adjusted_score > self.threshold.get_threshold();
        self.threshold.update(adjusted_score, is_anomaly);
        
        adjusted_score
    }
    
    fn compute_rcf_score(&mut self, value: f32) -> f32 {
        let mut total_depth = 0;
        
        for tree in self.trees.iter_mut() {
            let result = tree.insert_with_stats(value);
            total_depth += result.depth;
        }
        
        let avg_depth = total_depth as f32 / NUM_TREES as f32;
        
        // Инвертируем: маленькая глубина = аномалия
        let score = 1.0 - (avg_depth / MAX_TREE_DEPTH as f32);
        
        // Усиливаем нелинейно
        powf(score, 0.8).clamp(0.0, 1.0)
    }
    
    fn adjust_score_by_volatility(&self, score: f32, value: f32) -> f32 {
        // Вычисляем среднее и std по истории
        let mut sum = 0.0;
        let mut min_val = f32::MAX;
        let mut max_val = f32::MIN;
        
        for i in 0..200 {
            let v = self.value_history[i];
            sum += v;
            if v < min_val { min_val = v; }
            if v > max_val { max_val = v; }
        }
        
        let mean = sum / 200.0;
        let range = max_val - min_val;
        
        // Если значение выходит за пределы исторического диапазона
        if value < min_val - range * 0.1 || value > max_val + range * 0.1 {
            return score.max(0.85);  // Точно аномалия
        }
        
        // Если значение близко к среднему — снижаем скор
        let deviation = (value - mean).abs();
        let normalized_dev = deviation / (range + 0.001);
        
        if normalized_dev < 0.2 {
            // Близко к среднему — скорее всего норма
            score * 0.4
        } else if normalized_dev > 0.8 {
            // Далеко от среднего — усиливаем
            score.max(0.7)
        } else {
            score
        }
    }
    
    fn partial_reset(&mut self) {
        for i in 0..NUM_TREES / 3 {
            self.trees[i] = RcfTree::new();
        }
    }
    
    pub fn get_threshold(&self) -> f32 {
        self.threshold.get_threshold()
    }
    
    pub fn reset(&mut self) {
        self.trees = Box::new([
            RcfTree::new(), RcfTree::new(), RcfTree::new(), RcfTree::new(), RcfTree::new(),
            RcfTree::new(), RcfTree::new(), RcfTree::new(), RcfTree::new(), RcfTree::new(),
            RcfTree::new(), RcfTree::new(), RcfTree::new(), RcfTree::new(), RcfTree::new(),
            RcfTree::new(), RcfTree::new(), RcfTree::new(), RcfTree::new(), RcfTree::new(),
            RcfTree::new(), RcfTree::new(), RcfTree::new(), RcfTree::new(), RcfTree::new(),
        ]);
        self.threshold = AdaptiveThreshold::new(INITIAL_THRESHOLD, TARGET_FPR);
        self.processed_count = 0;
        self.value_history = [0.0; 200];
        self.history_idx = 0;
        self.history_filled = false;
    }
    
    pub fn processed_count(&self) -> usize {
        self.processed_count
    }
}