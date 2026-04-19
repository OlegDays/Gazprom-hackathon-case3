#![no_std]

pub mod rcf;
pub mod learning;
pub mod expert;
pub mod ensemble;

pub use ensemble::{Ensemble20, FrameResult};
pub use expert::SignalExpert;