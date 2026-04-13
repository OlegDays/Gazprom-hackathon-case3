// ============================================================
// signal_expert.rs - Эксперт для одного потока данных
// ============================================================
//! Один эксперт, закрепленный за своим потоком данных
//! Содержит ансамбль RCF деревьев и online learning

use crate::rcf::RcfTree;
use crate::learning::{AdaptiveBaseline, AdaptiveThreshold};

/// Количество деревьев в ансамбле эксперта
/// 10 деревьев дают хороший баланс точность/скорость
const NUM_TREES: usize = 10;

/// Начальный порог аномальности (0.7 = достаточно консервативно)
const INITIAL_THRESHOLD: f32 = 0.7;

/// Целевая частота ложных срабатываний (1%)
const TARGET_FPR: f32 = 0.01;

/// Максимальная глубина дерева для нормализации
const MAX_TREE_DEPTH: usize = 10;

/// Эксперт для анализа одного потока данных
pub struct SignalExpert {
    /// Ансамбль RCF деревьев
    trees: [RcfTree; NUM_TREES],
    
    /// Адаптивная базовая линия (для нормализации)
    baseline: AdaptiveBaseline,
    
    /// Адаптивный порог аномальности
    threshold: AdaptiveThreshold,
    
    /// Счетчик обработанных точек
    processed_count: usize,
}

impl SignalExpert {
    /// Создает нового эксперта
    #[inline(always)]
    pub fn new() -> Self {
        // Инициализируем деревья
        let trees = [(); NUM_TREES].map(|_| RcfTree::new());
        
        Self {
            trees,
            baseline: AdaptiveBaseline::new(),
            threshold: AdaptiveThreshold::new(INITIAL_THRESHOLD, TARGET_FPR),
            processed_count: 0,
        }
    }
    
    /// Обрабатывает одну точку данных
    /// Возвращает оценку аномальности в диапазоне [0.0, 1.0]
    /// 
    /// # Аргументы
    /// * `value` - сырое значение с датчика
    /// 
    /// # Возвращает
    /// * `f32` - оценка аномальности (чем выше, тем более аномальна точка)
    #[inline(always)]
    pub fn process(&mut self, value: f32) -> f32 {
        self.processed_count += 1;
        
        // 1. Обновляем базовую линию и нормализуем значение
        let normalized = self.baseline.update(value);
        
        // 2. Вычисляем оценку аномальности через ансамбль RCF
        let rcf_score = self.compute_rcf_score(normalized);
        
        // 3. Обновляем порог на основе результата
        let is_anomaly = rcf_score > self.threshold.get_threshold();
        let final_threshold = self.threshold.update(rcf_score, is_anomaly);
        
        // 4. Возвращаем скор (для внешнего использования)
        rcf_score
    }
    
    /// Вычисляет оценку аномальности через ансамбль деревьев
    #[inline(always)]
    fn compute_rcf_score(&mut self, value: f32) -> f32 {
        let mut total_depth = 0;
        
        // Обрабатываем каждое дерево
        for tree in &mut self.trees {
            let depth = tree.insert(value);
            total_depth += depth;
        }
        
        // Средняя глубина по всем деревьям
        let avg_depth = total_depth as f32 / NUM_TREES as f32;
        
        // Нормализуем в оценку аномальности
        // Чем меньше глубина, тем выше оценка аномальности
        let max_depth = MAX_TREE_DEPTH as f32;
        let normalized_score = 1.0 - (avg_depth / max_depth).min(1.0);
        
        // Применяем нелинейное преобразование для усиления контраста
        // (аномалии становятся еще более заметными)
        let contrast_score = normalized_score.powf(1.5);
        
        contrast_score.clamp(0.0, 1.0)
    }
    
    /// Быстрая проверка, является ли значение аномальным
    /// Использует текущий порог эксперта
    #[inline(always)]
    pub fn is_anomaly(&self, score: f32) -> bool {
        score > self.threshold.get_threshold()
    }
    
    /// Возвращает текущий порог аномальности
    #[inline(always)]
    pub fn get_threshold(&self) -> f32 {
        self.threshold.get_threshold()
    }
    
    /// Сбрасывает эксперта в начальное состояние
    #[inline(always)]
    pub fn reset(&mut self) {
        *self = Self::new();
    }
    
    /// Возвращает количество обработанных точек
    #[inline(always)]
    pub fn processed_count(&self) -> usize {
        self.processed_count
    }
    
    /// Возвращает текущее среднее значение потока (для отладки)
    #[inline(always)]
    pub fn get_current_mean(&self) -> f32 {
        self.baseline.get_mean()
    }
    
    /// Возвращает текущее стандартное отклонение (для отладки)
    #[inline(always)]
    pub fn get_current_std(&self) -> f32 {
        self.baseline.get_std_dev()
    }
}

impl Default for SignalExpert {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_expert_normal_data() {
        let mut expert = SignalExpert::new();
        
        // Нормальные данные (синусоидальный сигнал)
        for i in 0..100 {
            let value = (i as f32 * 0.1).sin();
            let score = expert.process(value);
            
            // Оценка должна быть низкой для нормальных данных
            if i > 50 {
                assert!(score < 0.6);
            }
        }
    }
    
    #[test]
    fn test_expert_anomaly() {
        let mut expert = SignalExpert::new();
        
        // Сначала подаем нормальные данные
        for i in 0..50 {
            expert.process(10.0);
        }
        
        // Подаем аномалию
        let anomaly_score = expert.process(100.0);
        
        // Оценка аномалии должна быть высокой
        assert!(anomaly_score > 0.7);
    }
    
    #[test]
    fn test_expert_adaptation() {
        let mut expert = SignalExpert::new();
        
        // Имитируем изменение baseline (датчик дрейфует)
        for i in 0..200 {
            let value = 50.0 + (i as f32) * 0.5;  // Медленный дрейф
            let score = expert.process(value);
            
            // После адаптации оценка должна вернуться к норме
            if i > 150 {
                assert!(score < 0.6);
            }
        }
    }
}