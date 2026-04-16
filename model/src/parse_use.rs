// parse_use.rs
// Заготовка для main.rs, имеющая основу логики для парсинга данных из csv и передачи их в анализирующую систему
//#![no_std]
#![no_main]

extern crate libc;

use core::array::from_fn;
use core::ffi::CStr;
use core::fmt::Write;
use core::str;
use core::convert::TryInto;
use anomaly_detector::{Ensemble20};

mod parse;

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
    let path = CStr::from_bytes_with_nul(b"1.csv\0").unwrap();
    let mut file_descriptors = [0i32; parse::DATA_FIELDS_COUNT];
    let mut ensemble = Ensemble20::new();

    if !parse::parse_init(path.as_ptr()) {
        return 1;
    }

    if !parse::parse_header() {
        return 2;
    }
    
//    if !create_output_dir() {
//		return 3;
//    }

    // Массив для одной строки: [timestamp, field1, field2, ..., fieldN]
    let mut row = [0.0f32; parse::DATA_FIELDS_COUNT + 1];
    let mut idx: usize = 0;

    while parse::export_row(&mut row) {
		let mut values = [0.0; 20];
		values.copy_from_slice(&row[1..]); // копируем элементы с индекса 1 до конца (20 элементов)
		let result = ensemble.process_frame(&values);
        // здесь уже можно передавать точки в формате row[i], но row[0] это всегда значение timestamp.
    }
    
    for i in 0..parse::DATA_FIELDS_COUNT {
    unsafe {
        libc::close(file_descriptors[i]);
    }
}
    
    0
}
