// parse.rs
#![no_std]
#![allow(static_mut_refs)]

use core::ffi::c_char;

pub const MAX_CSV_SIZE: usize = 30 * 1024 * 1024;  // 30 МБ
pub const DATA_FIELDS_COUNT: usize = 20;          // количество полей данных (без timestamp)

static mut CSV_BUFFER: [u8; MAX_CSV_SIZE] = [0; MAX_CSV_SIZE];
static mut CSV_LEN: usize = 0;
static mut CURRENT_POS: usize = 0;
static mut HEADER_PARSED: bool = false;

struct HeaderField {
    start: usize,
    end: usize,
}
static mut HEADER_FIELDS: [HeaderField; DATA_FIELDS_COUNT] = [HeaderField { start: 0, end: 0 }; DATA_FIELDS_COUNT];

/// Инициализация: чтение файла и нормализация переводов строк
pub fn parse_init(path: *const c_char) -> bool {
    unsafe {
        let fd = libc::open(path, libc::O_RDONLY);
        if fd < 0 {
            return false;
        }

        let bytes_read = libc::read(fd, CSV_BUFFER.as_mut_ptr() as *mut libc::c_void, MAX_CSV_SIZE);
        let close_ok = libc::close(fd) == 0;
        if bytes_read < 0 || !close_ok {
            return false;
        }

        let mut len = bytes_read as usize;
        if len >= MAX_CSV_SIZE {
            return false;
        }

        // Нормализация: \r\n -> \n, удаление одиночных \r
        let mut wp = 0;
        let mut rp = 0;
        while rp < len {
            if rp + 1 < len && CSV_BUFFER[rp] == b'\r' && CSV_BUFFER[rp + 1] == b'\n' {
                CSV_BUFFER[wp] = b'\n';
                wp += 1;
                rp += 2;
            } else if CSV_BUFFER[rp] == b'\r' {
                rp += 1;
            } else {
                if wp != rp {
                    CSV_BUFFER[wp] = CSV_BUFFER[rp];
                }
                wp += 1;
                rp += 1;
            }
        }
        CSV_LEN = wp;
        CURRENT_POS = 0;
        HEADER_PARSED = false;
        true
    }
}

/// Парсинг заголовка (первой строки). Сохраняются имена полей данных (без timestamp).
pub fn parse_header() -> bool {
    unsafe {
        if HEADER_PARSED {
            return true;
        }
        if CSV_LEN == 0 {
            return false;
        }

        let mut pos = 0;
        // Пропускаем первое поле (timestamp) до ';'
        while pos < CSV_LEN && CSV_BUFFER[pos] != b';' && CSV_BUFFER[pos] != b'\n' {
            pos += 1;
        }
        if pos >= CSV_LEN || CSV_BUFFER[pos] != b';' {
            return false;
        }
        pos += 1; // перешагиваем ';'

        let mut idx = 0;
        while idx < DATA_FIELDS_COUNT && pos < CSV_LEN && CSV_BUFFER[pos] != b'\n' {
            // Пропускаем начальные пробелы
            while pos < CSV_LEN && CSV_BUFFER[pos] == b' ' {
                pos += 1;
            }
            let start = pos;
            while pos < CSV_LEN && CSV_BUFFER[pos] != b';' && CSV_BUFFER[pos] != b'\n' {
                pos += 1;
            }
            let end = pos;
            if start == end {
                return false; // пустое имя поля
            }
            HEADER_FIELDS[idx] = HeaderField { start, end };
            idx += 1;

            if pos < CSV_LEN && CSV_BUFFER[pos] == b';' {
                pos += 1;
            }
        }

        if idx != DATA_FIELDS_COUNT {
            return false;
        }

        // Переход к началу данных: ищем конец строки заголовка
        while pos < CSV_LEN && CSV_BUFFER[pos] != b'\n' {
            pos += 1;
        }
        if pos < CSV_LEN && CSV_BUFFER[pos] == b'\n' {
            pos += 1;
        }
        // Пропускаем возможные пустые строки после заголовка
        while pos < CSV_LEN && CSV_BUFFER[pos] == b'\n' {
            pos += 1;
        }
        CURRENT_POS = pos;
        HEADER_PARSED = true;
        true
    }
}

/// Возвращает имя i-го поля данных (0..DATA_FIELDS_COUNT-1) как &[u8]
pub fn get_header_field(i: usize) -> Option<&'static [u8]> {
    unsafe {
        if !HEADER_PARSED || i >= DATA_FIELDS_COUNT {
            return None;
        }
        let field = &HEADER_FIELDS[i];
        Some(&CSV_BUFFER[field.start..field.end])
    }
}

/// Подсчёт количества строк данных (если нужно). Должна вызываться после parse_header().
pub fn count_lines() -> usize {
    unsafe {
        if !HEADER_PARSED {
            return 0;
        }
        let mut pos = CURRENT_POS;
        let mut count = 0;
        let mut in_line = false;

        while pos < CSV_LEN {
            let b = CSV_BUFFER[pos];
            if b == b'\n' {
                if in_line {
                    count += 1;
                    in_line = false;
                }
            } else {
                in_line = true;
            }
            pos += 1;
        }
        // Если последняя строка не заканчивается \n, но содержит данные
        if in_line {
            count += 1;
        }
        count
    }
}

/// Извлекает timestamp и DATA_FIELDS_COUNT чисел из текущей строки.
/// Результат помещается в out: out[0] = timestamp, out[1..] = значения полей.
pub fn export_row(out: &mut [f32; DATA_FIELDS_COUNT + 1]) -> bool {
    unsafe {
        if !HEADER_PARSED || CURRENT_POS >= CSV_LEN {
            return false;
        }

        let mut pos = CURRENT_POS;
        let mut idx = 0;

        // 1. Парсим timestamp (первое поле)
        while pos < CSV_LEN && CSV_BUFFER[pos] == b' ' {
            pos += 1;
        }
        let start_ts = pos;
        while pos < CSV_LEN && CSV_BUFFER[pos] != b';' && CSV_BUFFER[pos] != b'\n' {
            pos += 1;
        }
        let end_ts = pos;
        if let Some(ts) = parse_f32(&CSV_BUFFER[start_ts..end_ts]) {
            out[0] = ts;
        } else {
            return false;
        }

        // Пропускаем ';' после timestamp
        if pos < CSV_LEN && CSV_BUFFER[pos] == b';' {
            pos += 1;
        } else {
            return false;
        }

        // 2. Парсим DATA_FIELDS_COUNT чисел
        while idx < DATA_FIELDS_COUNT && pos < CSV_LEN && CSV_BUFFER[pos] != b'\n' {
            // Пропускаем пробелы
            while pos < CSV_LEN && CSV_BUFFER[pos] == b' ' {
                pos += 1;
            }

            let start = pos;
            while pos < CSV_LEN && CSV_BUFFER[pos] != b';' && CSV_BUFFER[pos] != b'\n' {
                pos += 1;
            }
            let end = pos;

            if let Some(value) = parse_f32(&CSV_BUFFER[start..end]) {
                out[idx + 1] = value;
                idx += 1;
            } else {
                return false;
            }

            if pos < CSV_LEN && CSV_BUFFER[pos] == b';' {
                pos += 1;
            }
        }

        if idx != DATA_FIELDS_COUNT {
            return false;
        }

        // Переход к следующей строке
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

/// Парсит f32 из байтового слайса. Поддерживает десятичную точку, знак,
/// игнорирует точки-разделители тысяч.
fn parse_f32(s: &[u8]) -> Option<f32> {
    if s.is_empty() {
        return None;
    }

    // Находим последнюю точку (десятичную)
    let mut last_dot_pos = None;
    for (i, &b) in s.iter().enumerate() {
        if b == b'.' {
            last_dot_pos = Some(i);
        }
    }

    let mut i = 0;
    let len = s.len();

    let sign = if s[i] == b'-' {
        i += 1;
        -1.0
    } else {
        if s[i] == b'+' {
            i += 1;
        }
        1.0
    };

    let mut value = 0.0f32;
    let mut after_decimal = false;
    let mut divisor = 1.0;

    while i < len {
        let byte = s[i];
        if byte == b'.' {
            if Some(i) == last_dot_pos {
                after_decimal = true;
            }
            // иначе это разделитель тысяч – игнорируем
        } else if byte.is_ascii_digit() {
            let digit = (byte - b'0') as f32;
            if after_decimal {
                divisor *= 10.0;
                value += digit / divisor;
            } else {
                value = value * 10.0 + digit;
            }
        } else {
            return None;
        }
        i += 1;
    }

    Some(sign * value)
}