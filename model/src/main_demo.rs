use anomaly_detector::{Ensemble20, SignalExpert};

fn main() {
    println!("=== ДЕТЕКТОР АНОМАЛИЙ - РАСШИРЕННОЕ ТЕСТИРОВАНИЕ ===\n");
    
    // Тест 1: Только нормальные данные
    test_normal_only();
    
    // Тест 2: Смешанные данные с известными аномалиями
    test_mixed_with_anomalies();
    
    // Тест 3: Анализ одного эксперта в деталях
    test_single_expert_detail();
}

fn test_normal_only() {
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║  ТЕСТ 1: ТОЛЬКО НОРМАЛЬНЫЕ ДАННЫЕ                            ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");
    
    let mut ensemble = Ensemble20::new();
    
    // Обучаем на нормальных данных с вариациями
    println!("📊 Обучение на 200 нормальных значениях...");
    for i in 0..200 {
        let normal = 10.0 + (i as f32 * 0.1).sin() * 1.5;
        let values = [normal; 20];
        ensemble.process_frame(&values);
    }
    
    println!("\n📈 Тестирование 50 нормальных значений:\n");
    println!("  №  │ Значение │ Оценка  │ Порог  │ Статус");
    println!("─────┼──────────┼─────────┼────────┼─────────");
    
    let mut false_positives = 0;
    let total = 50;
    
    for i in 0..total {
        let normal = 10.0 + (i as f32 * 0.2).cos() * 1.2;
        let values = [normal; 20];
        let result = ensemble.process_frame(&values);
        
        if result.anomalies[0] {
            false_positives += 1;
        }
        
        let status = if result.anomalies[0] { "⚠️ FP " } else { "✅ OK " };
        println!(" {:3} │ {:8.2} │ {:7.3} │ {:6.3} │ {}", 
                 i + 1, normal, result.scores[0], ensemble.get_thresholds()[0], status);
    }
    
    let fpr = false_positives as f32 / total as f32 * 100.0;
    println!("\n📊 Результаты:");
    println!("   Ложные срабатывания: {}/{} ({:.1}%)", false_positives, total, fpr);
    println!("   Финальный порог: {:.3}", ensemble.get_thresholds()[0]);
}

fn test_mixed_with_anomalies() {
    println!("\n\n╔══════════════════════════════════════════════════════════════╗");
    println!("║  ТЕСТ 2: СМЕШАННЫЕ ДАННЫЕ С АНОМАЛИЯМИ                       ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");
    
    let mut ensemble = Ensemble20::new();
    
    // Фаза обучения
    println!("📊 Обучение на 100 нормальных значениях...");
    for i in 0..100 {
        let normal = 10.0 + (i as f32 * 0.05).sin() * 1.0;
        let values = [normal; 20];
        ensemble.process_frame(&values);
    }
    
    println!("\n📈 Тестирование смешанной последовательности:\n");
    println!("  №  │ Значение │ Тип       │ Оценка  │ Порог  │ Статус");
    println!("─────┼──────────┼───────────┼─────────┼────────┼─────────");
    
    // Предопределённая последовательность с аномалиями
    let test_sequence = [
        (10.2, false),   // норма
        (10.5, false),   // норма
        (25.0, true),    // АНОМАЛИЯ - сильный скачок
        (10.3, false),   // норма
        (9.8, false),    // норма
        (10.1, false),   // норма
        (15.0, true),    // АНОМАЛИЯ - средний скачок
        (10.4, false),   // норма
        (10.2, false),   // норма
        (30.0, true),    // АНОМАЛИЯ - очень сильный скачок
        (10.0, false),   // норма
        (10.5, false),   // норма
        (20.0, true),    // АНОМАЛИЯ - сильный скачок
        (10.1, false),   // норма
        (9.9, false),    // норма
        (10.3, false),   // норма
        (12.0, false),   // норма (лёгкое отклонение)
        (18.0, true),    // АНОМАЛИЯ - заметный скачок
        (10.2, false),   // норма
        (10.0, false),   // норма
    ];
    
    let mut true_positives = 0;
    let mut false_positives = 0;
    let mut true_negatives = 0;
    let mut false_negatives = 0;
    let mut total_anomalies = 0;
    let mut total_normals = 0;
    
    for (i, (value, is_real_anomaly)) in test_sequence.iter().enumerate() {
        let values = [*value; 20];
        let result = ensemble.process_frame(&values);
        let detected = result.anomalies[0];
        
        // Подсчёт статистики
        if *is_real_anomaly {
            total_anomalies += 1;
            if detected {
                true_positives += 1;
            } else {
                false_negatives += 1;
            }
        } else {
            total_normals += 1;
            if detected {
                false_positives += 1;
            } else {
                true_negatives += 1;
            }
        }
        
        let type_str = if *is_real_anomaly { "🔴 АНОМАЛИЯ" } else { "🟢 НОРМА   " };
        let status = if detected {
            if *is_real_anomaly { "✅ TP" } else { "⚠️ FP" }
        } else {
            if *is_real_anomaly { "❌ FN" } else { "✅ TN" }
        };
        
        println!(" {:3} │ {:8.2} │ {} │ {:7.3} │ {:6.3} │ {}", 
                 i + 1, value, type_str, result.scores[0], ensemble.get_thresholds()[0], status);
    }
    
    println!("\n📊 Метрики качества:");
    println!("   True Positives:  {}/{}", true_positives, total_anomalies);
    println!("   False Negatives: {}/{}", false_negatives, total_anomalies);
    println!("   True Negatives:  {}/{}", true_negatives, total_normals);
    println!("   False Positives: {}/{}", false_positives, total_normals);
    
    let precision = if true_positives + false_positives > 0 {
        true_positives as f32 / (true_positives + false_positives) as f32 * 100.0
    } else { 0.0 };
    
    let recall = if total_anomalies > 0 {
        true_positives as f32 / total_anomalies as f32 * 100.0
    } else { 0.0 };
    
    let f1 = if precision + recall > 0.0 {
        2.0 * precision * recall / (precision + recall)
    } else { 0.0 };
    
    println!("\n   Precision: {:.1}%", precision);
    println!("   Recall:    {:.1}%", recall);
    println!("   F1-Score:  {:.1}%", f1);
}

fn test_single_expert_detail() {
    println!("\n\n╔══════════════════════════════════════════════════════════════╗");
    println!("║  ТЕСТ 3: ДЕТАЛЬНЫЙ АНАЛИЗ С ГЛУБИНАМИ                        ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");
    
    let mut expert = SignalExpert::new();
    
    println!("📊 Обучение эксперта на 200 нормальных значениях (10.0 ± 1.5)...\n");
    
    // УВЕЛИЧИВАЕМ ОБУЧЕНИЕ ДО 200 (как в Тесте 1)
    for i in 0..200 {
        let normal = 10.0 + (i as f32 * 0.1).sin() * 1.5;
        expert.process(normal);
    }
    
    println!("📈 Тестирование с выводом глубины:\n");
    println!("  Значение │ Глубина │ Оценка  │ Порог  │ Статус      │ Комментарий");
    println!("───────────┼─────────┼─────────┼────────┼─────────────┼──────────────────");
    
    let test_values = [
        (9.5, "Норма, нижняя граница"),
        (10.0, "Норма, центр"),
        (10.5, "Норма, верхняя граница"),
        (12.0, "Лёгкое отклонение"),
        (15.0, "Средняя аномалия"),
        (20.0, "Сильная аномалия"),
        (30.0, "Очень сильная аномалия"),
        (10.2, "Возврат к норме"),
    ];
    
    for (value, comment) in test_values {
        let (score, avg_depth) = expert.process_with_depth(value);
        let threshold = expert.get_threshold();
        let detected = score > threshold;
        
        let status = if detected {
            "🔴 АНОМАЛИЯ"
        } else {
            "🟢 НОРМА   "
        };
        
        println!(" {:9.2} │ {:7.1} │ {:7.3} │ {:6.3} │ {} │ {}", 
                 value, avg_depth, score, threshold, status, comment);
    }
    
    println!("\n📊 Статистика эксперта:");
    println!("   Обработано значений: {}", expert.processed_count());
    println!("   Финальный порог: {:.3}", expert.get_threshold());
}