#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::trace_start;

#[unsafe(no_mangle)]
pub fn main() -> i32 {
    let ret = trace_start();
    println!("trace_start: {}", ret);
    ret as i32
}
