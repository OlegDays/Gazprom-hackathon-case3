// main.rs
#![no_std]
#![no_main]
#![allow(static_mut_refs)]

extern crate libc;

use core::ffi::CStr;
use core::ffi::c_char;
use core::panic::PanicInfo;
use anomaly_detector::Ensemble20;

mod parse;
mod output;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unsafe {
        libc::write(2, b"Panic occurred\n\0".as_ptr() as *const libc::c_void, 15);
        libc::exit(1);
    }
}

#[no_mangle]
pub extern "C" fn main(argc: i32, argv: *const *const c_char) -> i32 {
	// Проверяем, что передан ровно один аргумент (имя файла)
    if argc < 2 {
        unsafe {
            libc::write(2, b"Usage: program <input.csv>\n\0".as_ptr() as *const libc::c_void, 27);
        }
        return 1;
    }
    
    // Получаем argv[1]
    let path_cstr = unsafe {
        let arg_ptr = *argv.add(1);
        if arg_ptr.is_null() {
            libc::write(2, b"Invalid argument\n\0".as_ptr() as *const libc::c_void, 17);
            return 1;
        }
        CStr::from_ptr(arg_ptr)
    };

    if !parse::parse_init(path_cstr.as_ptr()) {
        return 1;
    }

    if !parse::parse_header() {
		return 2;
    }
    
    if !output::create_output_dir() {
		return 3;
    }
    
    let mut file_descriptors = [0i32; parse::DATA_FIELDS_COUNT];
    
    for i in 0..parse::DATA_FIELDS_COUNT {
        // Получаем имя поля из CSV-заголовка
        let field_name = match parse::get_header_field(i) {
            Some(name) => name,
            None => return 4,
        };
        let fd = output::open_output_file(field_name, i);
        if fd < 0 {
            return 5;
        }
        file_descriptors[i] = fd;
        // Записываем заголовок в файл
        if !output::write_header(fd, field_name) {
            return 6;
        }
    }
    
    let ensemble = Ensemble20::init();
    
    while let Some((ts_bytes, values)) = parse::export_row() {
		let result = ensemble.process_frame(&values);
		for i in 0..parse::DATA_FIELDS_COUNT {
			let is_good = !result.anomalies[i];
			if !output::write_row_with_timestamp(file_descriptors[i], ts_bytes, values[i], is_good) {
				return 7;
           }
		}
	}
    
    for i in 0..parse::DATA_FIELDS_COUNT {
		unsafe {libc::close(file_descriptors[i]);}
    }
    0
}
