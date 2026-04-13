use anomaly_detector::SignalExpert;

fn main() {
    println!("============================================================");
    println!("ДЕМОНСТРАЦИЯ ДЕТЕКТОРА АНОМАЛИЙ RCF");
    println!("============================================================");
    
    let mut expert = Box::new(SignalExpert::new());
    
    // ========== ЭТАП 1: Обучение на нормальных данных ==========
    println!("\n[ЭТАП 1] Обучение на нормальных данных (200 циклов)...");
    for i in 0..200 {
        let normal = (i as f32 * 0.1).sin() * 10.0;
        expert.process(normal);
        
        if i % 50 == 0 && i > 0 {
            println!("  Прогресс: {} циклов, порог: {:.3}", i, expert.get_threshold());
        }
    }
    println!("  Обучение завершено! Порог: {:.3}", expert.get_threshold());
    
    // ========== ЭТАП 2: Тест на нормальных данных ==========
    println!("\n[ЭТАП 2] Тест на нормальных данных (50 циклов)...");
    let mut normal_scores = [0.0; 50];
    for i in 0..50 {
        let normal = (i as f32 * 0.1).sin() * 10.0;
        let score = expert.process(normal);
        normal_scores[i] = score;
        
        if score > expert.get_threshold() {
            println!("  ⚠️  Ложное срабатывание на цикле {}: {:.3}", i, score);
        }
    }
    let mut sum = 0.0;
    for &s in &normal_scores {
        sum += s;
    }
    let avg_normal = sum / 50.0;
    println!("  Средняя оценка на нормальных данных: {:.3}", avg_normal);
    
    // ========== ЭТАП 3: Тест на аномалиях (отдельные) ==========
    println!("\n[ЭТАП 3] Тест на отдельных аномалиях:");
    let anomalies = [100.0, 200.0, -100.0, 500.0, 1000.0];
    for &anomaly in &anomalies {
        let score = expert.process(anomaly);
        let is_anom = if score > expert.get_threshold() { "ДА" } else { "НЕТ" };
        println!("  Значение: {:6.1} -> оценка: {:.3} -> аномалия: {}", anomaly, score, is_anom);
    }
    
    // ========== ЭТАП 4: Смешанные данные с аномалиями ==========
    println!("\n[ЭТАП 4] Тест на смешанных данных (1000 циклов)...");
    println!("  Аномалии составляют примерно 20% от данных");
    
    let total_cycles = 1000;
    let mut scores = [0.0; 1000];
    let mut true_positives = 0;
    let mut false_positives = 0;
    let anomaly_count = 200; // 20% от 1000
    
    // Генерируем позиции аномалий (каждая 5-я точка)
    for cycle in 0..total_cycles {
        let is_real_anomaly = (cycle % 5 == 0) && (cycle >= 100);
        
        // Базовое нормальное значение
        let base_value = (cycle as f32 * 0.05).sin() * 10.0;
        let mut value = base_value;
        
        if is_real_anomaly {
            // Аномалия: скачок
            value = base_value * 10.0;
        }
        
        let score = expert.process(value);
        scores[cycle] = score;
        
        let is_detected = score > expert.get_threshold();
        
        if is_real_anomaly && is_detected {
            true_positives += 1;
        } else if !is_real_anomaly && is_detected {
            false_positives += 1;
        }
        
        // Вывод каждые 100 циклов
        if cycle % 100 == 0 && cycle > 0 {
            let mut sum_100 = 0.0;
            for i in (cycle-100)..cycle {
                sum_100 += scores[i];
            }
            let avg = sum_100 / 100.0;
            println!("  Цикл {:4}: средняя оценка = {:.3}, порог = {:.3}", 
                     cycle, avg, expert.get_threshold());
        }
    }
    
    // ========== ИТОГОВАЯ СТАТИСТИКА ==========
    println!("\n============================================================");
    println!("ИТОГОВАЯ СТАТИСТИКА");
    println!("============================================================");
    
    let mut sum_all = 0.0;
    let mut max_score = 0.0;
    let mut min_score = 1.0;
    for &s in &scores {
        sum_all += s;
        if s > max_score { max_score = s; }
        if s < min_score { min_score = s; }
    }
    let avg_score_all = sum_all / total_cycles as f32;
    
    println!("\nОбщая статистика:");
    println!("  Всего циклов:           {}", total_cycles);
    println!("  Средняя оценка:         {:.3}", avg_score_all);
    println!("  Максимальная оценка:    {:.3}", max_score);
    println!("  Минимальная оценка:     {:.3}", min_score);
    println!("  Текущий порог:          {:.3}", expert.get_threshold());
    
    println!("\nОбнаружение аномалий:");
    println!("  Реальных аномалий:      {}", anomaly_count);
    println!("  Обнаружено аномалий:    {} из {} ({:.1}%)", 
             true_positives, anomaly_count, 
             true_positives as f32 / anomaly_count as f32 * 100.0);
    println!("  Ложных срабатываний:    {}", false_positives);
    
    let normal_count = total_cycles - anomaly_count;
    let false_positive_rate = false_positives as f32 / normal_count as f32 * 100.0;
    println!("  Частота ложных:         {:.2}%", false_positive_rate);
    
    // Метрики
    let precision = true_positives as f32 / (true_positives + false_positives) as f32;
    let recall = true_positives as f32 / anomaly_count as f32;
    let f1_score = 2.0 * precision * recall / (precision + recall);
    
    println!("\nМетрики качества:");
    println!("  Точность (Precision):   {:.3}", precision);
    println!("  Полнота (Recall):       {:.3}", recall);
    println!("  F1-мера:                {:.3}", f1_score);
    
    println!("\nВердикт:");
    if f1_score > 0.8 {
        println!("  Отлично! Детектор показывает высокое качество.");
    } else if f1_score > 0.6 {
        println!("  Хорошо! Детектор работает удовлетворительно.");
    } else if f1_score > 0.4 {
        println!("  Средне. Рекомендуется донастройка параметров.");
    } else {
        println!("  Плохо. Требуется серьезная настройка.");
    }
    
    // Динамика обучения
    let mut sum_first = 0.0;
    let mut sum_last = 0.0;
    for i in 0..100 {
        sum_first += scores[i];
        sum_last += scores[total_cycles - 100 + i];
    }
    let avg_first = sum_first / 100.0;
    let avg_last = sum_last / 100.0;
    
    println!("\nДинамика обучения:");
    println!("  Средняя оценка (первые 100):  {:.3}", avg_first);
    println!("  Средняя оценка (последние 100): {:.3}", avg_last);
    
    if avg_last < avg_first * 0.7 {
        println!("  Модель успешно обучилась.");
    } else {
        println!("  Обучение продолжается.");
    }
    
    println!("\n============================================================");
    println!("ДЕМОНСТРАЦИЯ ЗАВЕРШЕНА");
    println!("============================================================");
}