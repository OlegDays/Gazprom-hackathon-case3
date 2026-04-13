// ============================================================
// baseline.rs - Online learning для адаптивной базовой линии
// ============================================================
//! Адаптивная базовая линия для каждого потока данных
//! Использует экспоненциальное скользящее среднее (EMA)

/// Скорость адаптации среднего значения (0.01 = очень медленно, 0.5 = очень быстро)
/// Для промышленных датчиков рекомендуется 0.05-0.1
const MEAN_LEARNING_RATE: f32 = 0.1;

/// Скорость адаптации стандартного отклонения
const STD_LEARNING_RATE: f32 = 0.05;

/// Минимальное количество точек для активации online learning
const MIN_SAMPLES_FOR_BASELINE: usize = 30;

/// Максимальное значение стандартного отклонения (защита от выбросов)
const MAX_STD_DEV: f32 = 100.0;

/// Минимальное значение стандартного отклонения (защита от деления на ноль)
const MIN_STD_DEV: f32 = 0.001;

/// Размер буфера истории для вычисления статистики
/// Используем фиксированный массив для zero-allocation
const HISTORY_BUFFER_SIZE: usize = 100;

/// Адаптивная базовая линия для одного потока данных
pub struct AdaptiveBaseline {
    /// Текущее среднее значение (экспоненциальное скользящее среднее)
    mean: f32,
    
    /// Текущее стандартное отклонение
    std_dev: f32,
    
    /// Буфер истории для периодического пересчета
    history: [f32; HISTORY_BUFFER_SIZE],
    
    /// Позиция в буфере истории
    history_pos: usize,
    
    /// Количество накопленных точек в истории
    history_count: usize,
    
    /// Количество обработанных точек всего
    total_samples: usize,
}

impl AdaptiveBaseline {
    /// Создает новую адаптивную базовую линию
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            mean: 0.0,
            std_dev: 1.0,  // Начинаем с единицы, чтобы избежать деления на ноль
            history: [0.0; HISTORY_BUFFER_SIZE],
            history_pos: 0,
            history_count: 0,
            total_samples: 0,
        }
    }
    
    /// Обновляет базовую линию новым значением
    /// Возвращает нормализованное значение
    #[inline(always)]
    pub fn update(&mut self, value: f32) -> f32 {
        self.total_samples += 1;
        
        // Сохраняем в историю
        self.history[self.history_pos] = value;
        self.history_pos = (self.history_pos + 1) % HISTORY_BUFFER_SIZE;
        if self.history_count < HISTORY_BUFFER_SIZE {
            self.history_count += 1;
        }
        
        // Если накоплено достаточно данных, обновляем baseline
        if self.history_count >= MIN_SAMPLES_FOR_BASELINE {
            // Периодически (раз в 10 точек) делаем полный пересчет
            if self.total_samples % 10 == 0 {
                self.full_recalc();
            } else {
                // Быстрое обновление через EMA
                self.fast_update(value);
            }
        }
        
        // Возвращаем нормализованное значение
        self.normalize(value)
    }
    
    /// Быстрое обновление через экспоненциальное скользящее среднее
    #[inline(always)]
    fn fast_update(&mut self, value: f32) {
        // Обновляем среднее
        let delta = value - self.mean;
        self.mean += delta * MEAN_LEARNING_RATE;
        
        // Обновляем дисперсию (через абсолютное отклонение для робастности)
        let abs_delta = delta.abs();
        let std_delta = abs_delta - self.std_dev;
        self.std_dev += std_delta * STD_LEARNING_RATE;
        
        // Ограничиваем значения
        self.std_dev = self.std_dev.clamp(MIN_STD_DEV, MAX_STD_DEV);
    }
    
    /// Полный пересчет по буферу истории (более точный, но реже)
    #[inline(always)]
    fn full_recalc(&mut self) {
        if self.history_count == 0 {
            return;
        }
        
        // Вычисляем среднее
        let mut sum = 0.0;
        for i in 0..self.history_count {
            sum += self.history[i];
        }
        let new_mean = sum / self.history_count as f32;
        
        // Вычисляем стандартное отклонение
        let mut sum_sq = 0.0;
        for i in 0..self.history_count {
            let diff = self.history[i] - new_mean;
            sum_sq += diff * diff;
        }
        let new_std = (sum_sq / self.history_count as f32).sqrt();
        
        // Плавное обновление (сглаживание резких изменений)
        self.mean = self.mean * 0.7 + new_mean * 0.3;
        self.std_dev = self.std_dev * 0.7 + new_std * 0.3;
        self.std_dev = self.std_dev.clamp(MIN_STD_DEV, MAX_STD_DEV);
    }
    
    /// Нормализует значение относительно текущей базовой линии
    #[inline(always)]
    pub fn normalize(&self, value: f32) -> f32 {
        (value - self.mean) / self.std_dev
    }
    
    /// Возвращает текущий порог аномальности (среднее + 3 сигмы)
    #[inline(always)]
    pub fn get_upper_threshold(&self) -> f32 {
        self.mean + 3.0 * self.std_dev
    }
    
    /// Возвращает текущий порог аномальности (среднее - 3 сигмы)
    #[inline(always)]
    pub fn get_lower_threshold(&self) -> f32 {
        self.mean - 3.0 * self.std_dev
    }
    
    /// Проверяет, является ли значение аномальным по базовой линии
    #[inline(always)]
    pub fn is_outlier(&self, value: f32, sigma: f32) -> bool {
        let normalized = self.normalize(value);
        normalized.abs() > sigma
    }
    
    /// Сбрасывает базовую линию (для переинициализации)
    #[inline(always)]
    pub fn reset(&mut self) {
        *self = Self::new();
    }
    
    /// Возвращает текущее среднее (для отладки)
    #[inline(always)]
    pub fn get_mean(&self) -> f32 {
        self.mean
    }
    
    /// Возвращает текущее стандартное отклонение (для отладки)
    #[inline(always)]
    pub fn get_std_dev(&self) -> f32 {
        self.std_dev
    }
}

impl Default for AdaptiveBaseline {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

/// Адаптивный порог аномальности
/// Плавно подстраивается под частоту срабатываний
pub struct AdaptiveThreshold {
    /// Текущий порог (0.0 - 1.0)
    threshold: f32,
    
    /// Целевая частота ложных срабатываний (0.01 = 1%)
    target_false_positive_rate: f32,
    
    /// Скорость адаптации порога
    adaptation_rate: f32,
    
    /// Счетчик срабатываний для калибровки
    anomaly_count: usize,
    
    /// Общий счетчик точек
    total_count: usize,
}

impl AdaptiveThreshold {
    /// Создает новый адаптивный порог
    #[inline(always)]
    pub fn new(initial_threshold: f32, target_fpr: f32) -> Self {
        Self {
            threshold: initial_threshold,
            target_false_positive_rate: target_fpr,
            adaptation_rate: 0.001,  // Очень медленная адаптация
            anomaly_count: 0,
            total_count: 0,
        }
    }
    
    /// Обновляет порог на основе результата обнаружения
    #[inline(always)]
    pub fn update(&mut self, score: f32, is_anomaly: bool) -> f32 {
        self.total_count += 1;
        if is_anomaly {
            self.anomaly_count += 1;
        }
        
        // Периодическая калибровка (раз в 1000 точек)
        if self.total_count >= 1000 {
            let actual_fpr = self.anomaly_count as f32 / self.total_count as f32;
            
            // Корректируем порог
            if actual_fpr > self.target_false_positive_rate {
                // Слишком много аномалий - повышаем порог
                self.threshold += self.adaptation_rate;
            } else if actual_fpr < self.target_false_positive_rate * 0.5 {
                // Слишком мало аномалий - снижаем порог
                self.threshold -= self.adaptation_rate;
            }
            
            // Ограничиваем порог
            self.threshold = self.threshold.clamp(0.5, 0.95);
            
            // Сбрасываем счетчики для следующего периода
            self.anomaly_count = 0;
            self.total_count = 0;
        }
        
        self.threshold
    }
    
    /// Возвращает текущий порог
    #[inline(always)]
    pub fn get_threshold(&self) -> f32 {
        self.threshold
    }
    
    /// Устанавливает новый порог (ручная настройка)
    #[inline(always)]
    pub fn set_threshold(&mut self, threshold: f32) {
        self.threshold = threshold.clamp(0.1, 0.99);
    }
    
    /// Сбрасывает порог к начальному значению
    #[inline(always)]
    pub fn reset(&mut self, initial_threshold: f32) {
        self.threshold = initial_threshold;
        self.anomaly_count = 0;
        self.total_count = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_adaptive_baseline() {
        let mut baseline = AdaptiveBaseline::new();
        
        // Вставляем нормальные данные (распределенные вокруг 100)
        for i in 0..100 {
            let value = 100.0 + (i as f32 - 50.0) * 0.1;
            let normalized = baseline.update(value);
            // Нормализованные значения должны быть около 0
            assert!(normalized.abs() < 3.0);
        }
        
        // Проверяем, что среднее приблизилось к 100
        let mean = baseline.get_mean();
        assert!((mean - 100.0).abs() < 5.0);
        
        // Аномальное значение должно дать большую нормализацию
        let anomaly_normalized = baseline.normalize(200.0);
        assert!(anomaly_normalized > 5.0);
    }
    
    #[test]
    fn test_adaptive_threshold() {
        let mut threshold = AdaptiveThreshold::new(0.7, 0.01);
        
        // Симулируем нормальную работу (редкие аномалии)
        for _ in 0..1000 {
            let score = 0.5;  // Норма
            let is_anomaly = score > threshold.get_threshold();
            threshold.update(score, is_anomaly);
        }
        
        // Порог должен немного снизиться из-за отсутствия аномалий
        assert!(threshold.get_threshold() < 0.7);
        
        // Симулируем поток аномалий
        for _ in 0..1000 {
            let score = 0.9;  // Аномалия
            let is_anomaly = score > threshold.get_threshold();
            threshold.update(score, is_anomaly);
        }
        
        // Порог должен повыситься
        assert!(threshold.get_threshold() > 0.65);
    }
}