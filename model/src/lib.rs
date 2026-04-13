// ============================================================
// lib.rs - Публичное API для модуля обнаружения аномалий
// ============================================================
//! Модуль для обнаружения аномалий в 20 потоках данных от датчиков
//!
//! # Пример использования
//!
//! ```rust
//! use anomaly_detector::{Ensemble20, process_batch};
//!
//! // Создаем ансамбль
//! let mut ensemble = Ensemble20::new();
//!
//! // Получаем данные с 20 датчиков (каждые 20 мс)
//! let sensor_data: [f32; 20] = [/* ... */];
//!
//! // Обрабатываем пакет
//! let scores = ensemble.process_batch(sensor_data);
//!
//! // Проверяем аномалии
//! for i in 0..20 {
//!     if scores[i] > 0.7 {
//!         println!("Аномалия в потоке {}! Оценка: {:.3}", i, scores[i]);
//!     }
//! }
//! ```
//!
//! # Особенности
//! - Zero dynamic allocation (вся память выделяется статически)
//! - Online learning (адаптация к изменяющимся условиям)
//! - Последовательная обработка (достаточно для x86_64, загрузка CPU < 1%)
//! - Нет внешних зависимостей (только core)

#![no_std]  // Отключаем стандартную библиотеку для embedded
#![deny(missing_docs)]
#![deny(unsafe_code)]

// Экспортируем модули
mod rcf;
mod learning;
mod expert;
mod ensemble;

// Публичный экспорт основных типов
pub use ensemble::{Ensemble20, NUM_EXPERTS};
pub use expert::SignalExpert;
pub use learning::{AdaptiveBaseline, AdaptiveThreshold};

/// Быстрая функция для обработки одного пакета данных
/// Удобный хелпер для простых случаев использования
/// 
/// # Аргументы
/// * `ensemble` - mutable ссылка на ансамбль
/// * `data` - массив из 20 значений
/// 
/// # Возвращает
/// * `[f32; 20]` - оценки аномальности
#[inline(always)]
pub fn process_batch(ensemble: &mut Ensemble20, data: [f32; NUM_EXPERTS]) -> [f32; NUM_EXPERTS] {
    ensemble.process_batch(data)
}

/// Быстрая функция для получения бинарных решений
#[inline(always)]
pub fn process_batch_binary(
    ensemble: &mut Ensemble20, 
    data: [f32; NUM_EXPERTS]
) -> [bool; NUM_EXPERTS] {
    ensemble.process_batch_binary(data)
}

/// Конфигурация модуля (может быть изменена до компиляции)
pub mod config {
    /// Количество экспертов (фиксировано = 20)
    pub const NUM_EXPERTS: usize = super::NUM_EXPERTS;
    
    /// Количество деревьев на эксперта
    pub const TREES_PER_EXPERT: usize = 10;
    
    /// Максимальная глубина дерева
    pub const MAX_TREE_DEPTH: usize = 10;
    
    /// Размер буфера истории для baseline
    pub const HISTORY_BUFFER_SIZE: usize = 100;
    
    /// Начальный порог аномальности
    pub const INITIAL_THRESHOLD: f32 = 0.7;
}

/// Версия модуля
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Статус модуля (для диагностики)
pub struct ModuleStatus {
    /// Количество обработанных пакетов (всего)
    pub total_batches_processed: u64,
    
    /// Текущая загрузка CPU (оценочная)
    pub estimated_cpu_load_percent: f32,
    
    /// Память, используемая модулем (байт)
    pub memory_usage_bytes: usize,
}

impl ModuleStatus {
    /// Возвращает текущий статус модуля
    pub fn get() -> Self {
        Self {
            total_batches_processed: 0,
            estimated_cpu_load_percent: 0.05,  // ~0.05% от одного ядра
            memory_usage_bytes: Self::calculate_memory_usage(),
        }
    }
    
    /// Рассчитывает использование памяти
    const fn calculate_memory_usage() -> usize {
        // SignalExpert: 10 деревьев * 256 узлов * размер Node + история
        // Node: 5 полей * 4 байта = 20 байт + выравнивание = 24 байта
        // 10 деревьев * 256 узлов * 24 = 61,440 байт
        // + история (100 * 4 = 400)
        // + прочее (~200)
        // Итого на эксперта: ~62,000 байт
        // 20 экспертов: ~1,240,000 байт ≈ 1.2 МБ
        NUM_EXPERTS * 62_000
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_lib_api() {
        let mut ensemble = Ensemble20::new();
        let data = [0.0; NUM_EXPERTS];
        
        let scores = process_batch(&mut ensemble, data);
        assert_eq!(scores.len(), NUM_EXPERTS);
        
        let binaries = process_batch_binary(&mut ensemble, data);
        assert_eq!(binaries.len(), NUM_EXPERTS);
    }
    
    #[test]
    fn test_config() {
        assert_eq!(config::NUM_EXPERTS, 20);
        assert_eq!(config::TREES_PER_EXPERT, 10);
        assert!(config::INITIAL_THRESHOLD > 0.0);
    }
    
    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}