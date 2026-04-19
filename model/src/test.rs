use anomaly_detector::Ensemble20;
use std::time::Instant;

fn main() {
    println!("╔══════════════════════════════════════════════════════════════════════════════╗");
    println!("║                    ПРОМЫШЛЕННЫЙ НАГРУЗОЧНЫЙ ТЕСТ                              ║");
    println!("║                 ДЕТЕКТОР АНОМАЛИЙ НА RCF - 20 ПОТОКОВ                         ║");
    println!("╚══════════════════════════════════════════════════════════════════════════════╝\n");

    // ==================== ТЕСТ 1: ДЛИТЕЛЬНОЕ ОБУЧЕНИЕ ====================
    println!("┌──────────────────────────────────────────────────────────────────────────────┐");
    println!("│  ТЕСТ 1: ДЛИТЕЛЬНОЕ ОБУЧЕНИЕ НА НОРМАЛЬНЫХ ДАННЫХ (10,000 ОБРАЗЦОВ)          │");
    println!("└──────────────────────────────────────────────────────────────────────────────┘\n");

    let mut ensemble = Ensemble20::init();
    ensemble.reset();

    let start = Instant::now();

    for i in 0..10_000 {
        let mut values = [0.0; 20];
        for channel in 0..20 {
            let base = 10.0 + (channel as f32) * 0.5;
            let amp_factor = ((channel % 3) as f32) * 0.5;
            let amplitude = 1.0 + amp_factor;
            let noise = (i as f32 * 0.01 + channel as f32).sin() * 0.2;
            values[channel] = base + (i as f32 * 0.001).sin() * amplitude + noise;
        }
        ensemble.process_frame(&values);

        if i > 0 && i % 1000 == 0 {
            print!("  █▓▒░ Обучение: {:5}/10000 образцов\r", i);
        }
    }
    let training_time = start.elapsed();
    println!("  ✅ Обучение завершено за {:?}", training_time);
    println!("  📊 Среднее время на образец: {:.2} мкс\n", 
             training_time.as_micros() as f32 / 10_000.0);

    // ==================== ТЕСТ 2: СТАБИЛЬНОСТЬ НА НОРМЕ ====================
    println!("┌──────────────────────────────────────────────────────────────────────────────┐");
    println!("│  ТЕСТ 2: ПРОВЕРКА СТАБИЛЬНОСТИ НА 5,000 НОРМАЛЬНЫХ ОБРАЗЦАХ                  │");
    println!("└──────────────────────────────────────────────────────────────────────────────┘\n");

    ensemble.reset();
    let mut false_positives = [0; 20];
    let mut scores_sum = [0.0; 20];
    let mut scores_min = [f32::MAX; 20];
    let mut scores_max = [f32::MIN; 20];

    let start = Instant::now();

    for i in 0..5_000 {
        let mut values = [0.0; 20];
        for channel in 0..20 {
            let base = 10.0 + (channel as f32) * 0.5;
            let amp_factor = ((channel % 3) as f32) * 0.5;
            let amplitude = 1.0 + amp_factor;
            values[channel] = base + (i as f32 * 0.002).cos() * amplitude;
        }
        let result = ensemble.process_frame(&values);

        for channel in 0..20 {
            if result.anomalies[channel] {
                false_positives[channel] += 1;
            }
            scores_sum[channel] += result.scores[channel];
            if result.scores[channel] < scores_min[channel] {
                scores_min[channel] = result.scores[channel];
            }
            if result.scores[channel] > scores_max[channel] {
                scores_max[channel] = result.scores[channel];
            }
        }

        if i > 0 && i % 1000 == 0 {
            print!("  █▓▒░ Тестирование: {:5}/5000 образцов\r", i);
        }
    }
    let test_time = start.elapsed();
    println!("  ✅ Тестирование завершено за {:?}", test_time);

    println!("\n  📊 Статистика по каналам:");
    println!("  ┌────────┬──────────┬──────────┬──────────┬──────────┬──────────┐");
    println!("  │ Канал  │ FP       │ FP Rate  │ Min Score│ Max Score│ Avg Score│");
    println!("  ├────────┼──────────┼──────────┼──────────┼──────────┼──────────┤");

    let mut total_fp = 0;
    for channel in 0..20 {
        let fp_rate = false_positives[channel] as f32 / 5_000.0 * 100.0;
        let avg_score = scores_sum[channel] / 5_000.0;
        total_fp += false_positives[channel];
        println!("  │ {:6} │ {:8} │ {:7.2}% │ {:8.3} │ {:8.3} │ {:8.3} │",
                 channel + 1, false_positives[channel], fp_rate,
                 scores_min[channel], scores_max[channel], avg_score);
    }
    println!("  └────────┴──────────┴──────────┴──────────┴──────────┴──────────┘");

    let overall_fpr = total_fp as f32 / (20.0 * 5_000.0) * 100.0;
    println!("\n  📈 Общий False Positive Rate: {:.2}% ({}/100000)", overall_fpr, total_fp);
    println!("  ⏱️  Среднее время на кадр: {:.2} мкс\n", 
             test_time.as_micros() as f32 / 5_000.0);

    // ==================== ТЕСТ 3: ДЕТЕКТИРОВАНИЕ АНОМАЛИЙ ====================
    println!("┌──────────────────────────────────────────────────────────────────────────────┐");
    println!("│  ТЕСТ 3: ДЕТЕКТИРОВАНИЕ РАЗЛИЧНЫХ ТИПОВ АНОМАЛИЙ                             │");
    println!("└──────────────────────────────────────────────────────────────────────────────┘\n");

    let anomaly_tests = [
        ("Внезапный скачок (+50%)", 0.95, 1),
        ("Внезапное падение (-50%)", 0.95, 2),
        ("Медленный дрейф вверх", 0.80, 3),
        ("Высокочастотный шум", 0.70, 4),
        ("Заморозка сигнала", 0.60, 5),
    ];

    for (test_idx, (name, expected_rate, test_type)) in anomaly_tests.iter().enumerate() {
        println!("  🧪 Тест {}.{}: {}", test_idx + 1, name, name);
        ensemble.reset();

        // Предобучение на 500 нормальных образцах
        for i in 0..2000 {
            let mut values = [0.0; 20];
            for channel in 0..20 {
                let base = 10.0 + (channel as f32) * 0.5;
                values[channel] = base + (i as f32 * 0.01).sin();
            }
            ensemble.process_frame(&values);
        }

        let test_samples = 200;
        let mut detections = 0;
        for i in 0..test_samples {
            let mut values = [0.0; 20];
            for channel in 0..20 {
                let base = 10.0 + (channel as f32) * 0.5;
                let normal_value = base + (i as f32 * 0.01).sin();
                if channel % 5 == 0 {
                    values[channel] = match test_type {
                        1 => normal_value * 1.5,
                        2 => normal_value * 0.5,
                        3 => normal_value + (i as f32 * 0.01),
                        4 => normal_value + (i as f32 * 10.0).sin() * 3.0,
                        5 => 10.0,
                        _ => normal_value,
                    };
                } else {
                    values[channel] = normal_value;
                }
            }
            let result = ensemble.process_frame(&values);
            for &ch in &[0, 5, 10, 15] {
                if result.anomalies[ch] {
                    detections += 1;
                }
            }
        }
        let detection_rate = detections as f32 / (4.0 * test_samples as f32);
        let status = if detection_rate >= *expected_rate { "✅" }
                     else if detection_rate >= expected_rate * 0.7 { "⚠️" }
                     else { "❌" };
        println!("     Детектировано: {}/{} ({:.1}%) {}",
                 detections, 4 * test_samples, detection_rate * 100.0, status);
        println!("     Ожидалось: ≥{:.0}%\n", expected_rate * 100.0);
    }

    // ==================== ТЕСТ 4: ПРОИЗВОДИТЕЛЬНОСТЬ ====================
    println!("┌──────────────────────────────────────────────────────────────────────────────┐");
    println!("│  ТЕСТ 4: СТРЕСС-ТЕСТ ПРОИЗВОДИТЕЛЬНОСТИ                                      │");
    println!("└──────────────────────────────────────────────────────────────────────────────┘\n");

    let iterations = [100, 1_000, 10_000];
    println!("  ┌──────────────┬──────────────┬──────────────┬──────────────┐");
    println!("  │ Образцов     │ Время (ms)   │ Образцов/с   │ CPU Load*    │");
    println!("  ├──────────────┼──────────────┼──────────────┼──────────────┤");

    for &n in &iterations {
        ensemble.reset();
        let start = Instant::now();
        for i in 0..n {
            let mut values = [0.0; 20];
            for channel in 0..20 {
                values[channel] = 10.0 + (i as f32 * 0.01 + channel as f32).sin();
            }
            ensemble.process_frame(&values);
        }
        let elapsed = start.elapsed();
        let samples_per_sec = n as f64 / elapsed.as_secs_f64();
        let cpu_load = (elapsed.as_secs_f64() / 0.02) / n as f64 * 100.0;
        println!("  │ {:12} │ {:12.2} │ {:12.0} │ {:12.1}% │",
                 n, elapsed.as_millis(), samples_per_sec, cpu_load);
    }
    println!("  └──────────────┴──────────────┴──────────────┴──────────────┘");
    println!("  * при условии цикла обработки 20 мс\n");

    // ==================== ТЕСТ 5: ОДНОВРЕМЕННЫЕ АНОМАЛИИ ====================
    println!("┌──────────────────────────────────────────────────────────────────────────────┐");
    println!("│  ТЕСТ 5: ОДНОВРЕМЕННЫЕ АНОМАЛИИ НА НЕСКОЛЬКИХ КАНАЛАХ                        │");
    println!("└──────────────────────────────────────────────────────────────────────────────┘\n");

    ensemble.reset();
    for i in 0..1_000 {
        let mut values = [0.0; 20];
        for channel in 0..20 {
            values[channel] = 10.0 + (i as f32 * 0.01 + channel as f32).sin();
        }
        ensemble.process_frame(&values);
    }

    // Тест 5.1
    let mut values = [10.0; 20];
    values[0] = 25.0;
    let result = ensemble.process_frame(&values);
    println!("  📍 Тест 5.1: Аномалия на одном канале");
    println!("     Канал 0 (аномалия): {} (score: {:.3})",
             if result.anomalies[0] { "🔴 ДЕТЕКТИРОВАНО" } else { "🟢 НОРМА" }, result.scores[0]);
    println!("     Канал 1 (норма):    {} (score: {:.3})",
             if result.anomalies[1] { "🔴 АНОМАЛИЯ" } else { "🟢 НОРМА" }, result.scores[1]);

    // Тест 5.2
    let mut values = [10.0; 20];
    for &ch in &[0, 4, 8, 12, 16] { values[ch] = 30.0; }
    let result = ensemble.process_frame(&values);
    println!("\n  📍 Тест 5.2: Аномалии на 5 каналах одновременно");
    let mut detected = 0;
    for &ch in &[0, 4, 8, 12, 16] {
        if result.anomalies[ch] { detected += 1; }
        println!("     Канал {:2} (аномалия): {} (score: {:.3})",
                 ch, if result.anomalies[ch] { "🔴 ДЕТЕКТИРОВАНО" } else { "🟢 НОРМА" }, result.scores[ch]);
    }
    println!("     Детектировано {} из 5 аномалий", detected);

    // Тест 5.3
    let mut values = [10.0; 20];
    values[0] = 15.0;  values[5] = 20.0; values[10] = 30.0; values[15] = 50.0;
    let result = ensemble.process_frame(&values);
    println!("\n  📍 Тест 5.3: Аномалии разной силы");
    println!("     Канал 0  (15.0 - слабая):     {} (score: {:.3})",
             if result.anomalies[0] { "🔴 ДЕТЕКТИРОВАНО" } else { "🟢 НОРМА" }, result.scores[0]);
    println!("     Канал 5  (20.0 - средняя):    {} (score: {:.3})",
             if result.anomalies[5] { "🔴 ДЕТЕКТИРОВАНО" } else { "🟢 НОРМА" }, result.scores[5]);
    println!("     Канал 10 (30.0 - сильная):    {} (score: {:.3})",
             if result.anomalies[10] { "🔴 ДЕТЕКТИРОВАНО" } else { "🟢 НОРМА" }, result.scores[10]);
    println!("     Канал 15 (50.0 - оч.сильная): {} (score: {:.3})",
             if result.anomalies[15] { "🔴 ДЕТЕКТИРОВАНО" } else { "🟢 НОРМА" }, result.scores[15]);
}