// parse.rs
#![allow(static_mut_refs)]

use core::ffi::c_char;

pub const MAX_CSV_SIZE: usize = 30 * 1024 * 1024;  // 30 МБ
pub const DATA_FIELDS_COUNT: usize = 20;  // количество полей данных (без timestamp)
// Максимальная длина имени поля в UTF-8 (исходное ≤ 255 байт в CP1251 → ≤ 510 байт в UTF-8)
const MAX_FIELD_UTF8_LEN: usize = 256;

static mut CSV_BUFFER: [u8; MAX_CSV_SIZE] = [0; MAX_CSV_SIZE];
static mut CSV_LEN: usize = 0;
static mut CURRENT_POS: usize = 0;
static mut HEADER_PARSED: bool = false;
static mut HEADER_UTF8: [[u8; MAX_FIELD_UTF8_LEN]; DATA_FIELDS_COUNT] = [[0; MAX_FIELD_UTF8_LEN]; DATA_FIELDS_COUNT];
static mut HEADER_UTF8_LEN: [usize; DATA_FIELDS_COUNT] = [0; DATA_FIELDS_COUNT];

#[derive(Copy, Clone)]
pub struct HeaderField {
    start: usize,
    end: usize,
}
static mut HEADER_FIELDS: [HeaderField; DATA_FIELDS_COUNT] = [HeaderField { start: 0, end: 0 }; DATA_FIELDS_COUNT];

// Инициализация: чтение файла и нормализация переводов строк
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

        let len = bytes_read as usize;
        if len >= MAX_CSV_SIZE {
            return false;
        }

        // Нормализация: \r\n превращается в \n, удаление одиночных \r
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
        // Удаляем BOM в начале файла, если он есть
		if CSV_LEN >= 3 && CSV_BUFFER[0] == 0xEF && CSV_BUFFER[1] == 0xBB && CSV_BUFFER[2] == 0xBF {
			// И сдвигаем данные на 3 байта влево
			let mut i = 0;
			while i + 3 < CSV_LEN {
				CSV_BUFFER[i] = CSV_BUFFER[i + 3];
				i += 1;
			}
			CSV_LEN -= 3;
		}
        true
    }
}

// Парсинг заголовка (первой строки).
pub fn parse_header() -> bool {
    unsafe {
        if HEADER_PARSED {
            return true;
        }
        if CSV_LEN == 0 {
            return false;
        }

        let mut pos = 0;
        while pos < CSV_LEN && CSV_BUFFER[pos] != b';' && CSV_BUFFER[pos] != b'\n' {
            pos += 1;
        }
        if pos >= CSV_LEN || CSV_BUFFER[pos] != b';' {
            return false;
        }
        pos += 1; // перешагиваем ';'

        let mut idx = 0;
        while idx < DATA_FIELDS_COUNT && pos < CSV_LEN && CSV_BUFFER[pos] != b'\n' {
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
        
        // Определяем кодировку: если весь заголовок корректный UTF-8 — считаем UTF-8, иначе CP1251.
		let mut is_utf8 = true;
		for idx in 0..DATA_FIELDS_COUNT {
			let field = &HEADER_FIELDS[idx];
			let slice = &CSV_BUFFER[field.start..field.end];
			// Проверяем, является ли слайс корректным UTF-8
			if core::str::from_utf8(slice).is_err() {
				is_utf8 = false;
				break;
			}
		}

		// Конвертируем каждое поле в UTF-8 и сохраняем в HEADER_UTF8
		for idx in 0..DATA_FIELDS_COUNT {
			let field = &HEADER_FIELDS[idx];
			let src = &CSV_BUFFER[field.start..field.end];
			let dst = &mut HEADER_UTF8[idx];
			let mut dst_pos = 0;
			if is_utf8 {
				// Просто копируем байты (уже UTF-8)
				for &b in src {
					if dst_pos < MAX_FIELD_UTF8_LEN {
						dst[dst_pos] = b;
						dst_pos += 1;
					}
				}
			} else {
				// Конвертируем из CP1251
				for &b in src {
					let written = encode_utf8_from_cp1251(b, &mut dst[dst_pos..]);
					dst_pos += written;
					if dst_pos >= MAX_FIELD_UTF8_LEN {
						break;
					}
				}
			}
			HEADER_UTF8_LEN[idx] = dst_pos;
		}
        HEADER_PARSED = true;
        true
    }
}

// Возвращает имя i-го поля данных (0..DATA_FIELDS_COUNT-1) как &[u8]
pub fn get_header_field(i: usize) -> Option<&'static [u8]> {
    unsafe {
        if !HEADER_PARSED || i >= DATA_FIELDS_COUNT {
            return None;
        }
        let len = HEADER_UTF8_LEN[i];
        Some(&HEADER_UTF8[i][..len])
    }
}

// Извлекает timestamp и DATA_FIELDS_COUNT чисел из текущей строки.
// Результат помещается в out (out[0] = timestamp, out[1..] = значения полей).
pub fn export_row() -> Option<(&'static [u8], [f32; DATA_FIELDS_COUNT])> {
    unsafe {
        if !HEADER_PARSED || CURRENT_POS >= CSV_LEN {
            return None;
        }

        let mut pos = CURRENT_POS;
        let mut idx = 0;
        let mut values = [0.0f32; DATA_FIELDS_COUNT];

        let ts_start;
        let ts_end;

        // Пропускаем пробелы
        while pos < CSV_LEN && CSV_BUFFER[pos] == b' ' {
            pos += 1;
        }
        ts_start = pos;
        while pos < CSV_LEN && CSV_BUFFER[pos] != b';' && CSV_BUFFER[pos] != b'\n' {
            pos += 1;
        }
        ts_end = pos;
        let timestamp_bytes = &CSV_BUFFER[ts_start..ts_end];

        // Пропускаем ';' после timestamp
        if pos < CSV_LEN && CSV_BUFFER[pos] == b';' {
            pos += 1;
        } else {
            return None;
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
                values[idx] = value;
                idx += 1;
            } else {
                return None;
            }

            if pos < CSV_LEN && CSV_BUFFER[pos] == b';' {
                pos += 1;
            }
        }

        if idx != DATA_FIELDS_COUNT {
            return None;
        }

        // Переход к следующей строке
        while pos < CSV_LEN && CSV_BUFFER[pos] != b'\n' {
            pos += 1;
        }
        if pos < CSV_LEN && CSV_BUFFER[pos] == b'\n' {
            pos += 1;
        }
        CURRENT_POS = pos;

        Some((timestamp_bytes, values))
    }
}

// Парсит f32 из байтового слайса. Поддерживает десятичную точку, знак,
// игнорирует точки-разделители тысяч.
fn parse_f32(s: &[u8]) -> Option<f32> {
    if s.is_empty() {
        return None;
    }

    // Находим последнюю точку
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
            // иначе это разделитель тысяч
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

// Таблица соответствия байт Unicode (CP1251)
const CP1251_TO_UNICODE: [u16; 128] = [
    0x0402, 0x0403, 0x201A, 0x0453, 0x201E, 0x2026, 0x2020, 0x2021,
    0x20AC, 0x2030, 0x0409, 0x2039, 0x040A, 0x040C, 0x040B, 0x040F,
    0x0452, 0x2018, 0x2019, 0x201C, 0x201D, 0x2022, 0x2013, 0x2014,
    0x0098, 0x2122, 0x0459, 0x203A, 0x045A, 0x045C, 0x045B, 0x045F,
    0x00A0, 0x040E, 0x045E, 0x0408, 0x00A4, 0x0490, 0x00A6, 0x00A7,
    0x0401, 0x00A9, 0x0404, 0x00AB, 0x00AC, 0x00AD, 0x00AE, 0x0407,
    0x00B0, 0x00B1, 0x0406, 0x0456, 0x0491, 0x00B5, 0x00B6, 0x00B7,
    0x0451, 0x2116, 0x0454, 0x00BB, 0x0458, 0x0405, 0x0455, 0x0457,
    0x0410, 0x0411, 0x0412, 0x0413, 0x0414, 0x0415, 0x0416, 0x0417,
    0x0418, 0x0419, 0x041A, 0x041B, 0x041C, 0x041D, 0x041E, 0x041F,
    0x0420, 0x0421, 0x0422, 0x0423, 0x0424, 0x0425, 0x0426, 0x0427,
    0x0428, 0x0429, 0x042A, 0x042B, 0x042C, 0x042D, 0x042E, 0x042F,
    0x0430, 0x0431, 0x0432, 0x0433, 0x0434, 0x0435, 0x0436, 0x0437,
    0x0438, 0x0439, 0x043A, 0x043B, 0x043C, 0x043D, 0x043E, 0x043F,
    0x0440, 0x0441, 0x0442, 0x0443, 0x0444, 0x0445, 0x0446, 0x0447,
    0x0448, 0x0449, 0x044A, 0x044B, 0x044C, 0x044D, 0x044E, 0x044F,
];

// Преобразует байт CP1251 в последовательность UTF-8 в `dst`.
// Возвращает количество записанных байт.
fn encode_utf8_from_cp1251(byte: u8, dst: &mut [u8]) -> usize {
    if byte < 0x80 {
        dst[0] = byte;
        return 1;
    }
    let cp = CP1251_TO_UNICODE[(byte - 0x80) as usize] as u32;
    if cp < 0x800 {
        dst[0] = 0xC0 | ((cp >> 6) as u8);
        dst[1] = 0x80 | ((cp & 0x3F) as u8);
        2
    } else {
        dst[0] = 0xE0 | ((cp >> 12) as u8);
        dst[1] = 0x80 | (((cp >> 6) & 0x3F) as u8);
        dst[2] = 0x80 | ((cp & 0x3F) as u8);
        3
    }
}
