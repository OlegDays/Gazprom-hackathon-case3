#![no_std]

pub mod rcf {
    mod node;
    mod tree;
    pub use node::Node;
    pub use tree::RcfTree;
}

pub mod learning {
    mod baseline;
    pub use baseline::AdaptiveThreshold;
}

pub mod expert {
    mod signal_expert;
    pub use signal_expert::SignalExpert;
}

pub mod ensemble {
    mod ensemble_20;
    pub use ensemble_20::{Ensemble20, FrameResult};
}

pub use ensemble::Ensemble20;
pub use ensemble::FrameResult;

pub fn init_detector() {
    Ensemble20::new;
}