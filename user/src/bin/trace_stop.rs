#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::trace_stop;

#[unsafe(no_mangle)]
pub fn main() -> i32 {
    let ret = trace_stop();
    println!("trace_stop: {}", ret);
    ret as i32
}
