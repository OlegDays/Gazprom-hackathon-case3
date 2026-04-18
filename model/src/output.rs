// output.rs
// Модуль записи результатов в CSV-файлы (no_std + libc)

use core::ffi::c_char;
use core::str;

const OUTPUT_DIR: &[u8] = b"output\0";
const PERMISSIONS: libc::mode_t = 0o666;

/// Создаёт выходную директорию (если не существует)
pub fn create_output_dir() -> bool {
    unsafe {
        // mkdir возвращает 0 при успехе, -1 при ошибке; EEXIST допустимо
        libc::mkdir(OUTPUT_DIR.as_ptr() as *const c_char, 0o755);
        // Игнорируем ошибку (директория может уже быть)
        true
    }
}

/// Открывает файл для записи данных одного датчика.
/// Имя файла формируется как "output/<имя_поля>.csv",
/// недопустимые символы заменяются на '_'.
pub fn open_output_file(field_name: &[u8], _index: usize) -> i32 {
    let mut path_buf = [0u8; 256];
    let dir = OUTPUT_DIR;
    let suffix = b".csv";
    let mut pos = 0;

    // Копируем "output/"
    for &b in dir.iter().take(dir.len() - 1) {
        // -1 чтобы не копировать нуль-терминатор
        path_buf[pos] = b;
        pos += 1;
    }
    path_buf[pos] = b'/';
    pos += 1;

    // Копируем имя поля, заменяя опасные символы
    for &b in field_name {
        if b == b'/' || b == b'\\' || b == b'\0' || b == b';' {
            path_buf[pos] = b'_';
        } else {
            path_buf[pos] = b;
        }
        pos += 1;
        if pos >= path_buf.len() - 10 {
            break;
        }
    }

    // Добавляем ".csv"
    for &b in suffix {
        path_buf[pos] = b;
        pos += 1;
    }
    path_buf[pos] = 0;

    unsafe {
        libc::open(
            path_buf.as_ptr() as *const c_char,
            libc::O_WRONLY | libc::O_CREAT | libc::O_TRUNC,
            PERMISSIONS,
        )
    }
}

/// Записывает строку в файл (используется для заголовков и разделителей)
pub fn write_str(fd: i32, s: &str) -> bool {
    unsafe {
        let buf = s.as_bytes();
        let written = libc::write(fd, buf.as_ptr() as *const libc::c_void, buf.len());
        written == buf.len() as isize
    }
}

/// Записывает число f32 с 6 знаками после запятой
pub fn write_f32(fd: i32, value: f32) -> bool {
    let mut buf = [0u8; 32];
    let mut pos = 0;

    // Знак
    if value < 0.0 {
        buf[pos] = b'-';
        pos += 1;
    }
    let abs_val = value.abs();
    let int_part = abs_val.trunc() as u32;
    let frac_part = ((abs_val.fract() * 1_000_000.0).round() as u32) % 1_000_000;

    // Целая часть
    let mut temp = int_part;
    let mut digits = [0u8; 10];
    let mut i = 0;
    if temp == 0 {
        digits[i] = b'0';
        i += 1;
    } else {
        while temp > 0 {
            digits[i] = b'0' + (temp % 10) as u8;
            temp /= 10;
            i += 1;
        }
    }
    for j in (0..i).rev() {
        buf[pos] = digits[j];
        pos += 1;
    }

    // Десятичная точка
    buf[pos] = b'.';
    pos += 1;

    // Дробная часть (всегда 6 цифр)
    let mut frac = frac_part;
    let mut frac_digits = [0u8; 6];
    for k in (0..6).rev() {
        frac_digits[k] = b'0' + (frac % 10) as u8;
        frac /= 10;
    }
    for k in 0..6 {
        buf[pos] = frac_digits[k];
        pos += 1;
    }

    unsafe {
        let written = libc::write(fd, buf.as_ptr() as *const libc::c_void, pos);
        written == pos as isize
    }
}

/// Записывает заголовок CSV для одного датчика:
/// "TimeStamp;Value;Status (1=Good,0=Bad)\n"
pub fn write_header(fd: i32, field_name: &[u8]) -> bool {
    let part1 = "TimeStamp;";
    let part2 = ";0 - Bad, 1 - Good\n";

    if !write_str(fd, part1) {
        return false;
    }
    // Преобразуем field_name в &str (только ASCII)
    if let Ok(name_str) = str::from_utf8(field_name) {
        if !write_str(fd, name_str) {
            return false;
        }
    } else {
        // fallback
        if !write_str(fd, "unknown") {
            return false;
        }
    }
    write_str(fd, part2)
}

/// Записывает одну строку данных, используя переданный слайс байт для timestamp.
pub fn write_row_with_timestamp(fd: i32, timestamp: &[u8], raw_value: f32, is_good: bool) -> bool {
    // Пишем timestamp
    unsafe {
        let written = libc::write(fd, timestamp.as_ptr() as *const libc::c_void, timestamp.len());
        if written != timestamp.len() as isize {
            return false;
        }
    }
    // Разделитель
    if !write_str(fd, ";") {
        return false;
    }
    // Значение
    if !write_f32(fd, raw_value) {
        return false;
    }
    // Разделитель и флаг
    if !write_str(fd, ";") {
        return false;
    }
    let flag = if is_good { "1\n" } else { "0\n" };
    write_str(fd, flag)
}
