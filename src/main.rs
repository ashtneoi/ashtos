#![no_std]
#![no_builtins]
#![no_main]
#![feature(global_asm)]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        // do nothing
    }
}

global_asm!(r#"
    .global go

    .section .vector

    go:
        li x6, 0x13333
        li x7, 0x100000
        sw x6, 0(x7)
"#);
