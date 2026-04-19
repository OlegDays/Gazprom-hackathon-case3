// parse_use.rs
// Заготовка для main.rs, имеющая основу логики для парсинга данных из csv и передачи их в анализирующую систему
//#![no_std]
#![no_main]
#![allow(static_mut_refs)]

extern crate libc;

use core::array::from_fn;
use core::ffi::CStr;
use core::fmt::Write;
use core::str;
use core::convert::TryInto;
use anomaly_detector::{Ensemble20};

mod parse;
mod output;

struct HeaderField {
	start: usize,
    end: usize,
}

#[no_mangle]
pub extern "C" fn main() -> i32 {//argc: i32, argv: *const *const c_char) -> i32 {
	
//	    if argc < 2 {
        // Недостаточно аргументов: требуется имя CSV-файла
//        return 1;
//    }
    
//	let filename = unsafe { *argv.add(1) };
    let path = CStr::from_bytes_with_nul(b"2.csv\0").unwrap();

    if !parse::parse_init(path.as_ptr()) {
        return 1;
    }

    if !parse::parse_header() {
        return 2;
    }
    
    if !output::create_output_dir() {
		return 3;
    }
    
    let mut file_descriptors = [0i32; parse::DATA_FIELDS_COUNT];
    let mut idx: usize = 0;
    
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
    
    let mut ensemble = Ensemble20::init();
    ensemble.reset();
    
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
		unsafe {
			libc::close(file_descriptors[i]);
		}
    }
    
    0
}

/*
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
*/
