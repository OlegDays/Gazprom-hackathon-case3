// parse.rs
#![no_std]
#![allow(static_mut_refs)]

use core::ffi::c_char;

//Уточнить
const MAX_CSV_SIZE: usize = 30 * 1024 * 1024;
const DATA_FIELDS_COUNT: usize = 20;

static mut CSV_BUFFER: [u8; MAX_CSV_SIZE] = [0; MAX_CSV_SIZE];
static mut CSV_LEN: usize = 0;
static mut CURRENT_POS: usize = 0;
static mut HEADER_SKIPPED: bool = false;

/// Инициализация: читает файл, нормализует переводы строк, пропускает заголовок.
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

        // Нормализация концов строк: \r\n -> \n, одиночные \r -> удалить
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
        len = wp;
        CSV_LEN = len;
        CURRENT_POS = 0;

        // Пропуск заголовка (первой строки)
        while CURRENT_POS < CSV_LEN && CSV_BUFFER[CURRENT_POS] != b'\n' {
            CURRENT_POS += 1;
        }
        if CURRENT_POS < CSV_LEN && CSV_BUFFER[CURRENT_POS] == b'\n' {
            CURRENT_POS += 1;
        }

        // Пропускаем возможные пустые строки после заголовка
        while CURRENT_POS < CSV_LEN && CSV_BUFFER[CURRENT_POS] == b'\n' {
            CURRENT_POS += 1;
        }

        HEADER_SKIPPED = true;
        true
    }
}

/// Возвращает массив из 23 чисел очередной строки (timestamp пропускается).
pub fn export_row(out: &mut [f32; DATA_FIELDS_COUNT]) -> bool {
    unsafe {
        if !HEADER_SKIPPED || CURRENT_POS >= CSV_LEN {
            return false;
        }

        let mut pos = CURRENT_POS;
        let mut idx = 0;

        // Пропускаем timestamp (первое поле до ';')
        while pos < CSV_LEN && CSV_BUFFER[pos] != b';' && CSV_BUFFER[pos] != b'\n' {
            pos += 1;
        }
        if pos >= CSV_LEN || CSV_BUFFER[pos] != b';' {
            return false;
        }
        pos += 1; // перешагиваем ';'

        // Парсим x чисел
        while idx <DATA_FIELDS_COUNT && pos < CSV_LEN && CSV_BUFFER[pos] != b'\n' {
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
                out[idx] = value;
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

/// Парсит f32 из байтов, игнорируя точки-разделители тысяч.
/// Поддерживает десятичную точку, знак, целую и дробную часть.
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