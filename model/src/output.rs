const OUTPUT_DIR: &str = "output\0";
const PERMISSIONS: libc::mode_t = 0o666;

fn create_output_dir() -> bool {
    unsafe {
        libc::mkdir(OUTPUT_DIR.as_ptr() as *const libc::c_char, 0o755);
        // Ошибку игнорируем – папка может уже существовать
        true
    }
}

fn open_output_file(field_name: &[u8], index: usize) -> i32 {
    let mut path_buf = [0u8; 256];
    let dir = OUTPUT_DIR.as_bytes();
    let prefix = b"/";
    let suffix = b".csv";
    let mut pos = 0;
    // Копируем "output/"
    for &b in dir.iter().take(dir.len() - 1) { // убираем нуль-терминатор
        path_buf[pos] = b;
        pos += 1;
    }
    path_buf[pos] = b'/';
    pos += 1;
    // Копируем имя поля
    for &b in field_name {
        if b == b'/' || b == b'\\' || b == b'\0' {
            path_buf[pos] = b'_';
        } else {
            path_buf[pos] = b;
        }
        pos += 1;
        if pos >= path_buf.len() - 10 { break; }
    }
    // Добавляем .csv и нуль
    for &b in suffix {
        path_buf[pos] = b;
        pos += 1;
    }
    path_buf[pos] = 0;
    unsafe {
        libc::open(path_buf.as_ptr() as *const libc::c_char, libc::O_WRONLY | libc::O_CREAT | libc::O_TRUNC, PERMISSIONS)
    }
}

fn write_f32(fd: i32, value: f32) -> bool {
    let mut buf = [0u8; 32];
    let mut pos = 0;
    // Обработка знака
    if value < 0.0 {
        buf[pos] = b'-';
        pos += 1;
    }
    let abs_val = value.abs();
    let int_part = abs_val.trunc() as u32;
    let frac_part = ((abs_val.fract() * 1_000_000.0).round() as u32) % 1_000_000;
    // Записываем целую часть
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
    // Дробная часть с ведущими нулями
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
    // Убираем завершающие нули, если не нужно – оставляем 6 знаков для единообразия
    // Можно обрезать, но в задании не критично.
    unsafe {
        let written = libc::write(fd, buf.as_ptr() as *const libc::c_void, pos);
        written == pos as isize
    }
}

fn write_header(fd: i32, field_name: &[u8]) -> bool {
    let part1 = b"TimeStamp;";
    let part2 = b";0 - Bad, 1 - Good\n";
    if !write_str(fd, core::str::from_utf8(part1).unwrap()) {
        return false;
    }
    if !write_str(fd, core::str::from_utf8(field_name).unwrap_or("unknown")) {
        return false;
    }
    if !write_str(fd, core::str::from_utf8(part2).unwrap()) {
        return false;
    }
    true
}
