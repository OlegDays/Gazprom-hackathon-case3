// ============================================================
// global.rs - Глобальная статическая память для экспертов
// ============================================================

use crate::expert::SignalExpert;
use crate::ensemble::NUM_EXPERTS;
use core::cell::UnsafeCell;

/// Глобальный массив экспертов в статической памяти
static mut EXPERTS: [SignalExpert; NUM_EXPERTS] = unsafe {
    // Это небезопасно, но мы гарантируем инициализацию до первого использования
    core::mem::zeroed()
};

/// Инициализатор глобальных экспертов
pub struct GlobalEnsemble;

impl GlobalEnsemble {
    /// Инициализирует глобальных экспертов (вызвать один раз при старте)
    pub unsafe fn init() {
        for i in 0..NUM_EXPERTS {
            EXPERTS[i] = SignalExpert::new();
        }
    }
    
    /// Обрабатывает пакет данных
    pub fn process_batch(data: [f32; NUM_EXPERTS]) -> [f32; NUM_EXPERTS] {
        let mut results = [0.0; NUM_EXPERTS];
        unsafe {
            for i in 0..NUM_EXPERTS {
                // Временный хак - нужно сделать mutable доступ
                results[i] = (&mut EXPERTS[i] as *mut SignalExpert).as_mut().unwrap().process(data[i]);
            }
        }
        results
    }
}