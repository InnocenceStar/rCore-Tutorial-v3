//! The main module and entrypoint
//!
//! The operating system and app also starts in this module. Kernel code starts
//! executing from `entry.asm`, after which [`rust_main()`] is called to
//! initialize various pieces of functionality [`clear_bss()`]. (See its source code for
//! details.)
//!
//! We then call [`println!`] to display `Hello, world!`.

#![deny(missing_docs)]
#![deny(warnings)]
#![no_std]
#![no_main]

macro_rules! linker_symbol_addr {
    ($symbol:path) => {
        ($symbol as *const ()).addr()
    };
}

use core::arch::global_asm;
use log::*;

#[macro_use]
mod console;
mod lang_items;
mod logging;
mod sbi;

global_asm!(include_str!("entry.asm"));

/// clear BSS segment
pub fn clear_bss() {
    unsafe extern "C" {
        safe fn sbss();
        safe fn ebss();
    }
    (linker_symbol_addr!(sbss)..linker_symbol_addr!(ebss))
        .for_each(|a| unsafe { (a as *mut u8).write_volatile(0) });
}

/// the rust entry-point of os
#[unsafe(no_mangle)]
pub fn rust_main() -> ! {
    unsafe extern "C" {
        safe fn stext(); // begin addr of text segment
        safe fn etext(); // end addr of text segment
        safe fn srodata(); // start addr of Read-Only data segment
        safe fn erodata(); // end addr of Read-Only data ssegment
        safe fn sdata(); // start addr of data segment
        safe fn edata(); // end addr of data segment
        safe fn sbss(); // start addr of BSS segment
        safe fn ebss(); // end addr of BSS segment
        safe fn boot_stack_lower_bound(); // stack lower bound
        safe fn boot_stack_top(); // stack top
    }
    clear_bss();
    logging::init();
    println!("[kernel] Hello, world!");
    trace!(
        "[kernel] .text [{:#x}, {:#x})",
        linker_symbol_addr!(stext),
        linker_symbol_addr!(etext)
    );
    debug!(
        "[kernel] .rodata [{:#x}, {:#x})",
        linker_symbol_addr!(srodata),
        linker_symbol_addr!(erodata)
    );
    info!(
        "[kernel] .data [{:#x}, {:#x})",
        linker_symbol_addr!(sdata),
        linker_symbol_addr!(edata)
    );
    warn!(
        "[kernel] boot_stack top=bottom={:#x}, lower_bound={:#x}",
        linker_symbol_addr!(boot_stack_top),
        linker_symbol_addr!(boot_stack_lower_bound)
    );
    error!(
        "[kernel] .bss [{:#x}, {:#x})",
        linker_symbol_addr!(sbss),
        linker_symbol_addr!(ebss)
    );

    // CI autotest success: sbi::shutdown(false)
    // CI autotest failed : sbi::shutdown(true)
    sbi::shutdown(false)
}
