// ============================================================
// ensemble_20.rs - Ансамбль из 20 независимых экспертов
// ============================================================
//! Ансамбль для параллельной обработки 20 потоков данных
//!
//! Каждый эксперт закреплен за своим потоком и не пересекается с другими

use crate::expert::SignalExpert;

/// Количество экспертов (соответствует количеству потоков датчиков)
pub const NUM_EXPERTS: usize = 20;

/// Ансамбль из 20 экспертов для анализа 20 потоков данных
pub struct Ensemble20 {
    /// Фиксированный массив экспертов (без динамической памяти)
    experts: [SignalExpert; NUM_EXPERTS],
}

impl Ensemble20 {
    /// Создает новый ансамбль из 20 экспертов
    /// 
    /// # Пример
    /// ```
    /// let ensemble = Ensemble20::new();
    /// ```
    #[inline(always)]
    pub fn new() -> Self {
        // Инициализируем 20 независимых экспертов
        let experts = [(); NUM_EXPERTS].map(|_| SignalExpert::new());
        
        Self { experts }
    }
    
    /// Обрабатывает пакет из 20 точек данных (по одной на каждый поток)
    /// 
    /// # Аргументы
    /// * `data` - массив из 20 значений датчиков (каждый эксперт получает свою точку)
    /// 
    /// # Возвращает
    /// * `[f32; 20]` - массив оценок аномальности для каждого потока
    /// 
    /// # Пример
    /// ```
    /// let mut ensemble = Ensemble20::new();
    /// let data = [1.0, 2.0, 3.0, /* ... */];
    /// let scores = ensemble.process_batch(data);
    /// ```
    #[inline(always)]
    pub fn process_batch(&mut self, data: [f32; NUM_EXPERTS]) -> [f32; NUM_EXPERTS] {
        let mut results = [0.0; NUM_EXPERTS];
        
        // Последовательная обработка всех 20 экспертов
        // На x86_64 это занимает ~6-10 микросекунд
        for i in 0..NUM_EXPERTS {
            results[i] = self.experts[i].process(data[i]);
        }
        
        results
    }
    
    /// Обрабатывает пакет и возвращает бинарные решения (аномалия/норма)
    /// 
    /// # Аргументы
    /// * `data` - массив из 20 значений датчиков
    /// 
    /// # Возвращает
    /// * `[bool; 20]` - true если аномалия, false если норма
    #[inline(always)]
    pub fn process_batch_binary(&mut self, data: [f32; NUM_EXPERTS]) -> [bool; NUM_EXPERTS] {
        let mut results = [false; NUM_EXPERTS];
        
        for i in 0..NUM_EXPERTS {
            let score = self.experts[i].process(data[i]);
            results[i] = self.experts[i].is_anomaly(score);
        }
        
        results
    }
    
    /// Обрабатывает пакет и возвращает и оценки, и бинарные решения
    /// 
    /// # Возвращает
    /// * `([f32; 20], [bool; 20])` - (оценки, решения)
    #[inline(always)]
    pub fn process_batch_full(&mut self, data: [f32; NUM_EXPERTS]) -> ([f32; NUM_EXPERTS], [bool; NUM_EXPERTS]) {
        let mut scores = [0.0; NUM_EXPERTS];
        let mut decisions = [false; NUM_EXPERTS];
        
        for i in 0..NUM_EXPERTS {
            scores[i] = self.experts[i].process(data[i]);
            decisions[i] = self.experts[i].is_anomaly(scores[i]);
        }
        
        (scores, decisions)
    }
    
    /// Сбрасывает всех экспертов в начальное состояние
    /// (полезно при перезапуске системы)
    #[inline(always)]
    pub fn reset_all(&mut self) {
        for expert in &mut self.experts {
            expert.reset();
        }
    }
    
    /// Возвращает ссылку на эксперта по индексу
    #[inline(always)]
    pub fn get_expert(&self, index: usize) -> Option<&SignalExpert> {
        if index < NUM_EXPERTS {
            Some(&self.experts[index])
        } else {
            None
        }
    }
    
    /// Возвращает mutable ссылку на эксперта по индексу
    #[inline(always)]
    pub fn get_expert_mut(&mut self, index: usize) -> Option<&mut SignalExpert> {
        if index < NUM_EXPERTS {
            Some(&mut self.experts[index])
        } else {
            None
        }
    }
    
    /// Возвращает текущие пороги всех экспертов (для диагностики)
    #[inline(always)]
    pub fn get_all_thresholds(&self) -> [f32; NUM_EXPERTS] {
        let mut thresholds = [0.0; NUM_EXPERTS];
        for i in 0..NUM_EXPERTS {
            thresholds[i] = self.experts[i].get_threshold();
        }
        thresholds
    }
    
    /// Возвращает количество обработанных точек каждым экспертом
    #[inline(always)]
    pub fn get_all_processed_counts(&self) -> [usize; NUM_EXPERTS] {
        let mut counts = [0; NUM_EXPERTS];
        for i in 0..NUM_EXPERTS {
            counts[i] = self.experts[i].processed_count();
        }
        counts
    }
    
    /// Проверяет, есть ли аномалии хотя бы в одном потоке
    #[inline(always)]
    pub fn has_any_anomaly(&mut self, data: [f32; NUM_EXPERTS]) -> bool {
        for i in 0..NUM_EXPERTS {
            let score = self.experts[i].process(data[i]);
            if self.experts[i].is_anomaly(score) {
                return true;
            }
        }
        false
    }
    
    /// Проверяет, есть ли аномалии в конкретных потоках
    /// Возвращает битовую маску (u32, где бит i = 1 если аномалия)
    #[inline(always)]
    pub fn get_anomaly_mask(&mut self, data: [f32; NUM_EXPERTS]) -> u32 {
        let mut mask = 0u32;
        
        for i in 0..NUM_EXPERTS {
            let score = self.experts[i].process(data[i]);
            if self.experts[i].is_anomaly(score) {
                mask |= 1 << i;
            }
        }
        
        mask
    }
}

impl Default for Ensemble20 {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ensemble_creation() {
        let ensemble = Ensemble20::new();
        
        // Проверяем, что все эксперты созданы
        for i in 0..NUM_EXPERTS {
            assert!(ensemble.get_expert(i).is_some());
        }
        
        // Проверяем, что за пределами массива ничего нет
        assert!(ensemble.get_expert(NUM_EXPERTS).is_none());
    }
    
    #[test]
    fn test_process_batch() {
        let mut ensemble = Ensemble20::new();
        
        // Создаем тестовые данные (нормальные значения)
        let mut data = [0.0; NUM_EXPERTS];
        for i in 0..NUM_EXPERTS {
            data[i] = 10.0;  // Все потоки получают одинаковое нормальное значение
        }
        
        // Обрабатываем несколько пакетов для инициализации
        for _ in 0..50 {
            let scores = ensemble.process_batch(data);
            
            // Оценки должны быть низкими для нормальных данных
            for score in scores.iter() {
                assert!(*score < 0.6);
            }
        }
    }
    
    #[test]
    fn test_anomaly_detection() {
        let mut ensemble = Ensemble20::new();
        
        // Нормальные данные для всех потоков
        let mut normal_data = [10.0; NUM_EXPERTS];
        for _ in 0..50 {
            ensemble.process_batch(normal_data);
        }
        
        // Создаем аномалию в 5-м потоке
        let mut anomaly_data = [10.0; NUM_EXPERTS];
        anomaly_data[5] = 100.0;  // Резкий скачок
        
        // Проверяем обнаружение аномалии
        let has_anomaly = ensemble.has_any_anomaly(anomaly_data);
        assert!(has_anomaly);
        
        // Проверяем битовую маску
        let mask = ensemble.get_anomaly_mask(anomaly_data);
        assert_eq!(mask & (1 << 5), 1 << 5);  // 5-й бит должен быть установлен
    }
    
    #[test]
    fn test_reset_all() {
        let mut ensemble = Ensemble20::new();
        
        // Обрабатываем данные
        let data = [5.0; NUM_EXPERTS];
        for _ in 0..100 {
            ensemble.process_batch(data);
        }
        
        // Сохраняем пороги до сброса
        let thresholds_before = ensemble.get_all_thresholds();
        
        // Сбрасываем
        ensemble.reset_all();
        
        // Пороги после сброса должны быть начальными (0.7)
        let thresholds_after = ensemble.get_all_thresholds();
        for i in 0..NUM_EXPERTS {
            assert_ne!(thresholds_before[i], thresholds_after[i]);
            assert!((thresholds_after[i] - 0.7).abs() < 0.01);
        }
    }
    
    #[test]
    fn test_individual_expert_access() {
        let mut ensemble = Ensemble20::new();
        
        // Получаем mutable ссылку на первого эксперта и изменяем его состояние
        if let Some(expert) = ensemble.get_expert_mut(0) {
            expert.process(42.0);
        }
        
        // Проверяем, что изменение сохранилось
        let counts = ensemble.get_all_processed_counts();
        assert_eq!(counts[0], 1);
        assert_eq!(counts[1], 0);  // Второй эксперт не изменился
    }
}