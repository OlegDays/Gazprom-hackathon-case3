// parse.rs
#![no_std]

use core::ffi::c_char;

// Максимальный размер CSV-файла. Для увеличения или уменьшения менять первое число
const MAX_CSV_SIZE: usize = 10 * 1024 * 1024;

// Глобальный буфер для содержимого файла
static mut CSV_BUFFER: [u8; MAX_CSV_SIZE] = [0; MAX_CSV_SIZE];
// Фактическая длина прочитанных данных
static mut CSV_LEN: usize = 0;
// Текущая позиция в буфере (указывает на начало следующей строки)
static mut CURRENT_POS: usize = 0;
// Флаг, была ли пропущена строка заголовка
static mut HEADER_SKIPPED: bool = false;

/// Инициализация модуля: чтение CSV-файла в буфер и пропуск первой строки.
/// `path` — путь к файлу, оканчивающийся нулём (C-строка).
/// Возвращает `true` при успехе, `false` при ошибке (файл не найден, слишком большой и т.п.).
pub fn parse_init(path: *const c_char) -> bool {
    unsafe {
        // Открываем файл (системный вызов open)
        let fd = libc::open(path, libc::O_RDONLY);
        if fd < 0 {
            return false;
        }

        // Читаем данные в буфер
        let bytes_read = libc::read(fd, CSV_BUFFER.as_mut_ptr() as *mut libc::c_void, MAX_CSV_SIZE);
        libc::close(fd);

        if bytes_read < 0 {
            return false;
        }

        let bytes_read = bytes_read as usize;
        if bytes_read == MAX_CSV_SIZE {
            // Файл, возможно, больше буфера – считаем ошибкой
            return false;
        }

        CSV_LEN = bytes_read;
        CURRENT_POS = 0;

        // Пропускаем первую строку (заголовок)
        while CURRENT_POS < CSV_LEN && CSV_BUFFER[CURRENT_POS] != b'\n' {
            CURRENT_POS += 1;
        }
        if CURRENT_POS < CSV_LEN && CSV_BUFFER[CURRENT_POS] == b'\n' {
            CURRENT_POS += 1; // переходим к началу второй строки
        }
        HEADER_SKIPPED = true;

        true
    }
}

/// Парсит следующую строку CSV (23 числа с плавающей точкой, без временной метки).
/// `out` — массив, в который будут записаны значения.
/// Возвращает `true`, если строка была успешно обработана,
/// `false` — если достигнут конец данных или произошла ошибка.
pub fn export_row(out: &mut [f32; 23]) -> bool {
    unsafe {
        if !HEADER_SKIPPED || CURRENT_POS >= CSV_LEN {
            return false;
        }

        let mut pos = CURRENT_POS;
        let mut idx = 0;

        // --- Пропускаем первое поле (timestamp) ---
        // Ищем конец первого поля (до ';' или конца строки)
        while pos < CSV_LEN && CSV_BUFFER[pos] != b';' && CSV_BUFFER[pos] != b'\n' {
            pos += 1;
        }
        // Если достигли конца строки или файла – строка неполная
        if pos >= CSV_LEN || CSV_BUFFER[pos] != b';' {
            return false;
        }
        // Пропускаем разделитель ';'
        pos += 1;

        // --- Парсим следующие 23 числа ---
        while idx < 23 && pos < CSV_LEN && CSV_BUFFER[pos] != b'\n' {
            // Пропускаем возможные пробелы и возврат каретки
            while pos < CSV_LEN && (CSV_BUFFER[pos] == b' ' || CSV_BUFFER[pos] == b'\r') {
                pos += 1;
            }

            // Находим конец текущего поля (разделитель ';' или конец строки)
            let start = pos;
            while pos < CSV_LEN && CSV_BUFFER[pos] != b';' && CSV_BUFFER[pos] != b'\n' {
                pos += 1;
            }
            let end = pos;

            // Парсим число (с поддержкой тысяч через точки)
            if let Some(value) = parse_f32(&CSV_BUFFER[start..end]) {
                out[idx] = value;
                idx += 1;
            } else {
                // Ошибка парсинга – считаем всю строку некорректной
                return false;
            }

            // Пропускаем разделитель ';', если есть
            if pos < CSV_LEN && CSV_BUFFER[pos] == b';' {
                pos += 1;
            }
        }

        // Проверяем, что в строке оказалось ровно 23 числа
        if idx != 23 {
            return false;
        }

        // Перемещаем CURRENT_POS на начало следующей строки (за '\n')
        while pos < CSV_LEN && CSV_BUFFER[pos] != b'\n' {
            pos += 1;
        }
        if pos < CSV_LEN && CSV_BUFFER[pos] == b'\n' {
            pos += 1;
        }
        CURRENT_POS = pos;

        true
    }
}

/// Парсер f32 из байтового слайса.
/// Поддерживает:
/// - целые и дробные числа (десятичный разделитель – точка);
/// - разделители тысяч в виде точек (например, "2.917.776" → 2917776.0);
/// - ведущий знак '+'/'-'.
/// Возвращает `Some(f32)` при успехе, `None` при ошибке.
fn parse_f32(s: &[u8]) -> Option<f32> {
    if s.is_empty() {
        return None;
    }

    // Преобразуем слайс в строку для удобной обработки тысяч
    let s_str = core::str::from_utf8(s).ok()?;

    // Обработка разделителей тысяч (точек, не являющихся десятичной точкой)
    let processed = if s_str.contains('.') {
        let dot_positions: Vec<_> = s_str.match_indices('.').collect();
        if dot_positions.len() > 1 {
            // Несколько точек: удаляем все, кроме последней (последняя остаётся десятичной)
            let last_dot = dot_positions.last().unwrap().0;
            let mut cleaned = String::with_capacity(s_str.len());
            for (i, ch) in s_str.char_indices() {
                if ch == '.' && i != last_dot {
                    continue;
                }
                cleaned.push(ch);
            }
            cleaned
        } else {
            // Ровно одна точка – оставляем как есть
            s_str.to_string()
        }
    } else {
        s_str.to_string()
    };

    // Парсим обработанную строку как f32
    let mut i = 0;
    let chars: Vec<char> = processed.chars().collect();
    let len = chars.len();

    // Знак
    let sign = if chars[i] == '-' {
        i += 1;
        -1.0
    } else {
        if chars[i] == '+' {
            i += 1;
        }
        1.0
    };

    // Целая часть
    let mut value = 0.0f32;
    while i < len && chars[i].is_ascii_digit() {
        value = value * 10.0 + (chars[i] as u8 - b'0') as f32;
        i += 1;
    }

    // Дробная часть
    if i < len && chars[i] == '.' {
        i += 1;
        let mut frac = 0.0f32;
        let mut divisor = 1.0f32;
        while i < len && chars[i].is_ascii_digit() {
            frac = frac * 10.0 + (chars[i] as u8 - b'0') as f32;
            divisor *= 10.0;
            i += 1;
        }
        value += frac / divisor;
    }

    if i != len {
        // Остались необработанные символы – ошибка
        return None;
    }

    Some(sign * value)
}
