//! Полноценный тестовый набор для запуска в стандартной среде (WSL, Linux, MacOS, Windows).
//! Использует `std` для вывода в консоль, форматирования и измерения времени.

use std::time::Instant;

// Если ваш крейт называется не 'anomaly_detector', замените это имя.
use anomaly_detector::ensemble::{get_instance, FrameResult};

// --- Конфигурация тестов ---
const WARMUP_FRAMES: usize = 500;
const FPR_TEST_DURATION: usize = 10000;
const ANOMALY_TEST_DURATION: usize = 200;
const ENSEMBLE_SIZE: usize = 20;

// --- ANSI цвета для красивого вывода в терминале ---
const GREEN: &str = "\x1b[32m";
const RED: &str = "\x1b[31m";
const YELLOW: &str = "\x1b[33m";
const BLUE: &str = "\x1b[34m";
const RESET: &str = "\x1b[0m";


// --- Точка входа ---
pub fn main() {
    println!("\n{BLUE}--- Anomaly Detector Test Suite Initializing ---{RESET}", BLUE=BLUE, RESET=RESET);
    let start_time = Instant::now();

    let results = run_all_tests();
    print_results(&results);

    let duration = start_time.elapsed();
    println!("\nTotal test suite duration: {GREEN}{:?}{RESET}", duration, GREEN=GREEN, RESET=RESET);
}


// --- Структуры для хранения результатов ---
#[derive(Debug, Default, Clone, Copy)]
pub struct TestResult {
    pub name: &'static str,
    pub detected: usize,
    pub total: usize,
    pub success: bool,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct TestResults {
    pub fpr: f32,
    pub anomaly_tests: [TestResult; 5],
}


// --- PRNG для no_std (оставляем его для воспроизводимости) ---
struct Prng { state: u32 }
impl Prng {
    fn new(seed: u32) -> Self { Self { state: seed } }
    fn next(&mut self) -> u32 {
        self.state = self.state.wrapping_mul(1_103_515_245).wrapping_add(12_345);
        self.state
    }
    fn rand_f32(&mut self) -> f32 { (self.next() as i32) as f32 / i32::MAX as f32 }
}


// --- Генератор тестовых сигналов ---
struct SignalGenerator {
    prng: Prng,
    base_values: [f32; ENSEMBLE_SIZE],
}
impl SignalGenerator {
    fn new() -> Self {
        Self { prng: Prng::new(12345), base_values: [100.0; ENSEMBLE_SIZE] }
    }
    fn normal_frame(&mut self) -> [f32; ENSEMBLE_SIZE] {
        let mut frame = [0.0; ENSEMBLE_SIZE];
        for i in 0..ENSEMBLE_SIZE {
            frame[i] = self.base_values[i] + self.prng.rand_f32() * 0.5;
        }
        frame
    }
}


// --- Основная функция запуска тестов ---
pub fn run_all_tests() -> TestResults {
    println!("Running tests...");
    let mut results = TestResults::default();
    
    results.fpr = test_fpr();
    results.anomaly_tests[0] = test_spike();
    results.anomaly_tests[1] = test_dip();
    results.anomaly_tests[2] = test_drift();
    results.anomaly_tests[3] = test_noise();
    results.anomaly_tests[4] = test_freeze();

    println!("{GREEN}All tests completed.{RESET}", GREEN=GREEN, RESET=RESET);
    results
}

// --- Функция вывода результатов (теперь она работает!) ---
pub fn print_results(results: &TestResults) {
    println!("\n\n{BLUE}--- Test Results ---{RESET}", BLUE=BLUE, RESET=RESET);
    println!("─────────────────────────────────────────────────────────");
    
    println!("\n[Test 1: False Positive Rate]");
    let fpr_status_color = if results.fpr < 5.0 { GREEN } else { RED };
    let fpr_status_text = if results.fpr < 5.0 { "PASS" } else { "FAIL" };
    println!("  FPR: {:.2}% (Target: < 5.0%) -> {}{}{}", results.fpr, fpr_status_color, fpr_status_text, RESET);

    println!("\n[Test 2: Anomaly Type Detection]");
    for test in &results.anomaly_tests {
        let percentage = (test.detected as f32 / test.total as f32) * 100.0;
        
        let (status_color, status_text) = if percentage >= 80.0 {
            (GREEN, "PASS")
        } else if percentage >= 60.0 {
            (YELLOW, "WARN")
        } else {
            (RED, "FAIL")
        };

        println!(
            "  - {:<12}: {:>3}/{:<3} ({:>5.1}%) -> {}{}{}",
            test.name, test.detected, test.total, percentage, status_color, status_text, RESET
        );
    }
    println!("─────────────────────────────────────────────────────────");
}


// --- Реализации конкретных тестов ---
fn test_fpr() -> f32 {
    let ensemble = get_instance();
    ensemble.reset();
    let mut gen = SignalGenerator::new();
    for _ in 0..WARMUP_FRAMES { ensemble.process_frame(&gen.normal_frame()); }
    let mut anomaly_frames = 0;
    for _ in 0..FPR_TEST_DURATION {
        if ensemble.process_frame(&gen.normal_frame()).has_anomaly() {
            anomaly_frames += 1;
        }
    }
    (anomaly_frames as f32 / FPR_TEST_DURATION as f32) * 100.0
}
fn test_spike() -> TestResult { run_anomaly_test("Spike", |f, _| { f[0] *= 1.5; f[5] *= 1.5; }) }
fn test_dip() -> TestResult { run_anomaly_test("Dip", |f, _| { f[1] *= 0.5; f[6] *= 0.5; }) }
fn test_drift() -> TestResult {
    run_anomaly_test("Slow Drift", |f, s| {
        let factor = 1.0 + (s as f32 / ANOMALY_TEST_DURATION as f32) * 0.3;
        f[2] *= factor; f[7] *= factor;
    })
}
fn test_noise() -> TestResult {
    let mut prng = Prng::new(54321);
    run_anomaly_test("Noise", move |f, _| { f[3] += prng.rand_f32() * 15.0; f[8] += prng.rand_f32() * 15.0; })
}
fn test_freeze() -> TestResult { run_anomaly_test("Freeze", |f, _| { f[4] = 100.0; f[9] = 100.0; }) }

// --- Универсальный "раннер" для тестов аномалий ---
fn run_anomaly_test<F>(name: &'static str, mut anomaly_fn: F) -> TestResult
where F: FnMut(&mut [f32; ENSEMBLE_SIZE], usize),
{
    let ensemble = get_instance();
    ensemble.reset();
    let mut gen = SignalGenerator::new();
    for _ in 0..WARMUP_FRAMES { ensemble.process_frame(&gen.normal_frame()); }
    let mut detected_frames = 0;
    for i in 0..ANOMALY_TEST_DURATION {
        let mut frame = gen.normal_frame();
        anomaly_fn(&mut frame, i);
        if ensemble.process_frame(&frame).has_anomaly() { detected_frames += 1; }
    }
    TestResult {
        name, detected: detected_frames, total: ANOMALY_TEST_DURATION,
        success: detected_frames as f32 / ANOMALY_TEST_DURATION as f32 > 0.8,
    }
}
    