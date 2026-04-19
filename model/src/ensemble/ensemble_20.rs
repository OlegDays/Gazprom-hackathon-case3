// ensemble_20.rs
// ансамбль моделей, по одной на поток

use crate::rcf::node::Node;
use crate::rcf::tree::{MAX_NODES_PER_TREE, WINDOW_SIZE};
use crate::expert::signal_expert::SignalExpert;
use core::array::from_fn;
use core::slice;   // <-- добавлен импорт

// характеристики каналов
pub const NUM_CHANNELS: usize = 20;
pub const NUM_TREES_PER_CHANNEL: usize = 25;

pub const TOTAL_NODES: usize = NUM_CHANNELS * NUM_TREES_PER_CHANNEL * MAX_NODES_PER_TREE;
pub const TOTAL_WINDOWS: usize = NUM_CHANNELS * NUM_TREES_PER_CHANNEL * WINDOW_SIZE;

pub static mut GLOBAL_NODES: [Node; TOTAL_NODES] = [Node::new(); TOTAL_NODES];
pub static mut GLOBAL_WINDOWS: [f32; TOTAL_WINDOWS] = [0.0; TOTAL_WINDOWS];

pub static mut ENSEMBLE: core::mem::MaybeUninit<Ensemble20> = core::mem::MaybeUninit::uninit();

pub struct Ensemble20 {
    experts: [SignalExpert; NUM_CHANNELS],
}

impl Ensemble20 {
    pub fn init() -> &'static mut Self {
        unsafe {
            let ensemble_ptr = &raw mut ENSEMBLE;   // сырой указатель вместо as_mut_ptr()
            let experts = from_fn(|i| {
                let node_offset = i * NUM_TREES_PER_CHANNEL * MAX_NODES_PER_TREE;
                let window_offset = i * NUM_TREES_PER_CHANNEL * WINDOW_SIZE;
                let mut expert = SignalExpert::new(node_offset, window_offset, 0.05);
                expert.init(node_offset, window_offset);
                expert
            });
            (*ensemble_ptr).write(Ensemble20 { experts });
            &mut *(*ensemble_ptr).as_mut_ptr()
        }
    }

    pub fn process_frame(&mut self, values: &[f32; NUM_CHANNELS]) -> FrameResult {
        let mut scores = [0.0; NUM_CHANNELS];
        let mut anomalies = [false; NUM_CHANNELS];
        unsafe {
            let nodes_ptr = &raw mut GLOBAL_NODES as *mut Node;
            let windows_ptr = &raw mut GLOBAL_WINDOWS as *mut f32;
            // Создаём безопасные слайсы нужного размера
            let nodes_slice = slice::from_raw_parts_mut(nodes_ptr, TOTAL_NODES);
            let windows_slice = slice::from_raw_parts_mut(windows_ptr, TOTAL_WINDOWS);
            for i in 0..NUM_CHANNELS {
                scores[i] = self.experts[i].process(values[i], nodes_slice, windows_slice);
                anomalies[i] = scores[i] > self.experts[i].get_threshold();
            }
        }
        FrameResult { scores, anomalies }
    }

    pub fn reset(&mut self) {
        unsafe {
            let nodes_ptr = &raw mut GLOBAL_NODES as *mut Node;
            let windows_ptr = &raw mut GLOBAL_WINDOWS as *mut f32;
            let nodes_slice = slice::from_raw_parts_mut(nodes_ptr, TOTAL_NODES);
            for node in nodes_slice.iter_mut() {
                *node = Node::new();
            }
            let windows_slice = slice::from_raw_parts_mut(windows_ptr, TOTAL_WINDOWS);
            for w in windows_slice.iter_mut() {
                *w = 0.0;
            }
        }
        let mut node_offset = 0;
        let mut window_offset = 0;
        for i in 0..NUM_CHANNELS {
            self.experts[i] = SignalExpert::new(node_offset, window_offset, 0.05);
            self.experts[i].init(node_offset, window_offset);
            node_offset += NUM_TREES_PER_CHANNEL * MAX_NODES_PER_TREE;
            window_offset += NUM_TREES_PER_CHANNEL * WINDOW_SIZE;
        }
    }
}

pub struct FrameResult {
    pub scores: [f32; NUM_CHANNELS],
    pub anomalies: [bool; NUM_CHANNELS],
}
