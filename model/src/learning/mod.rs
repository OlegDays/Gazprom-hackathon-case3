// ============================================================
// learning/mod.rs - Модуль online learning
// ============================================================
//! Online learning для адаптации к изменяющимся условиям
//!
//! Этот модуль предоставляет:
//! - AdaptiveBaseline: адаптивная нормализация данных
//! - AdaptiveThreshold: автоматическая настройка порога

mod baseline;

pub use baseline::{AdaptiveBaseline, AdaptiveThreshold};