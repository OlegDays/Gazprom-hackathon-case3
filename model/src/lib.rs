// ============================================================
// lib.rs - Публичное API для модуля обнаружения аномалий
// ============================================================

// Экспортируем модули
pub mod rcf;
mod learning;
mod expert;
mod ensemble;

// Публичный экспорт основных типов
pub use ensemble::{Ensemble20, NUM_EXPERTS};
pub use expert::SignalExpert;
pub use learning::{AdaptiveBaseline, AdaptiveThreshold};

/// Быстрая функция для обработки одного пакета данных
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

/// Конфигурация модуля
pub mod config {
    /// Количество экспертов (20)
    pub const NUM_EXPERTS: usize = super::NUM_EXPERTS;
    /// Количество деревьев на эксперта
    pub const TREES_PER_EXPERT: usize = 10;
    /// Максимальная глубина дерева
    pub const MAX_TREE_DEPTH: usize = 10;
    /// Размер буфера истории
    pub const HISTORY_BUFFER_SIZE: usize = 100;
    /// Начальный порог аномальности
    pub const INITIAL_THRESHOLD: f32 = 0.7;
}

/// Версия модуля
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Статус модуля для диагностики
pub struct ModuleStatus {
    /// Количество обработанных пакетов
    pub total_batches_processed: u64,
    /// Оценочная загрузка CPU в процентах
    pub estimated_cpu_load_percent: f32,
    /// Использование памяти в байтах
    pub memory_usage_bytes: usize,
}

impl ModuleStatus {
    /// Возвращает текущий статус модуля
    pub fn get() -> Self {
        Self {
            total_batches_processed: 0,
            estimated_cpu_load_percent: 0.05,
            memory_usage_bytes: Self::calculate_memory_usage(),
        }
    }
    
    /// Рассчитывает использование памяти
    pub const fn calculate_memory_usage() -> usize {
        NUM_EXPERTS * 62_000
    }
}