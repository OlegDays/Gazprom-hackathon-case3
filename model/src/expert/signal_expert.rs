//! Эксперт для обработки одного потока сигнала

use crate::rcf::RcfTree;
use crate::learning::AdaptiveThreshold;
use libm::powf;

const NUM_TREES: usize = 10;
const INITIAL_THRESHOLD: f32 = 0.65;
const TARGET_FPR: f32 = 5.0;
const MAX_TREE_DEPTH: usize = 5;
const RESET_INTERVAL: usize = 2000;

pub struct SignalExpert {
    trees: [RcfTree; NUM_TREES],
    threshold: AdaptiveThreshold,
    processed_count: usize,
    
    // История сырых значений для калибровки
    value_history: [f32; 200],
    history_idx: usize,
    history_filled: bool,
    
    // Флаг инициализации
    initialized: bool,
    init_buffer: [f32; 10],
    init_count: usize,
}

impl SignalExpert {
    pub fn new() -> Self {
        let trees = [
            RcfTree::new(), RcfTree::new(), RcfTree::new(), RcfTree::new(), RcfTree::new(),
            RcfTree::new(), RcfTree::new(), RcfTree::new(), RcfTree::new(), RcfTree::new(),
            RcfTree::new(), RcfTree::new(), RcfTree::new(), RcfTree::new(), RcfTree::new(),
            RcfTree::new(), RcfTree::new(), RcfTree::new(), RcfTree::new(), RcfTree::new(),
            RcfTree::new(), RcfTree::new(), RcfTree::new(), RcfTree::new(), RcfTree::new(),
        ];
        
        Self {
            trees,
            threshold: AdaptiveThreshold::new(INITIAL_THRESHOLD, TARGET_FPR),
            processed_count: 0,
            value_history: [0.0; 200],
            history_idx: 0,
            history_filled: false,
            initialized: false,
            init_buffer: [0.0; 10],
            init_count: 0,
        }
    }
    
    pub fn process(&mut self, value: f32) -> f32 {
        self.processed_count += 1;
        
        // Инициализация: собираем первые 10 значений для заполнения истории
        if !self.initialized {
            self.init_buffer[self.init_count] = value;
            self.init_count += 1;
            
            if self.init_count == 10 {
                // Заполняем историю первыми значениями
                let mean = self.init_buffer.iter().sum::<f32>() / 10.0;
                for i in 0..200 {
                    self.value_history[i] = mean;
                }
                self.initialized = true;
                self.history_filled = true;
            }
            
            // Во время инициализации возвращаем низкий скор
            return 0.1;
        }
        
        // Сохраняем значение в историю
        self.value_history[self.history_idx] = value;
        self.history_idx = (self.history_idx + 1) % 200;
        
        // Периодический частичный сброс
        if self.processed_count % RESET_INTERVAL == 0 {
            self.partial_reset();
        }
        
        // 1. Вычисляем скор RCF
        let score = self.compute_rcf_score(value);
        
        // 2. Корректируем скор на основе локальной волатильности
        let adjusted_score = self.adjust_score_by_volatility(score, value);
        
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
        powf(score, 0.9).clamp(0.0, 1.0)
    }
    
    fn adjust_score_by_volatility(&self, score: f32, value: f32) -> f32 {
        // Вычисляем статистику по истории
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
        
        // Если диапазон слишком маленький (заморозка) — это аномалия
        if range < 0.001 {
            return score.max(0.85);
        }
        
        // Проверяем резкий скачок (изменение за 1 шаг)
        let prev_value = self.value_history[(self.history_idx + 199) % 200];
        let step = (value - prev_value).abs();
        let step_normalized = step / (range + 0.001);
        
        // Резкий скачок (>30% от исторического диапазона) — точно аномалия
        if step_normalized > 0.3 {
            return score.max(0.9);
        }
        
        // Выход за исторический диапазон — точно аномалия
        if value < min_val - range * 0.1 || value > max_val + range * 0.1 {
            return score.max(0.85);
        }
        
        // Вычисляем отклонение от среднего
        let deviation = (value - mean).abs();
        let normalized_dev = deviation / (range + 0.001);
        
        if normalized_dev < 0.15 {
            // Очень близко к среднему — небольшая корректировка вниз
            score * 0.7
        } else if normalized_dev > 0.6 {
            // Далеко от среднего — усиливаем
            score.max(0.75)
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
        self.trees = [
            RcfTree::new(), RcfTree::new(), RcfTree::new(), RcfTree::new(), RcfTree::new(),
            RcfTree::new(), RcfTree::new(), RcfTree::new(), RcfTree::new(), RcfTree::new(),
            RcfTree::new(), RcfTree::new(), RcfTree::new(), RcfTree::new(), RcfTree::new(),
            RcfTree::new(), RcfTree::new(), RcfTree::new(), RcfTree::new(), RcfTree::new(),
            RcfTree::new(), RcfTree::new(), RcfTree::new(), RcfTree::new(), RcfTree::new(),
        ];
        self.threshold = AdaptiveThreshold::new(INITIAL_THRESHOLD, TARGET_FPR);
        self.processed_count = 0;
        self.value_history = [0.0; 200];
        self.history_idx = 0;
        self.history_filled = false;
        self.initialized = false;
        self.init_buffer = [0.0; 10];
        self.init_count = 0;
    }
    
    pub fn processed_count(&self) -> usize {
        self.processed_count
    }
}