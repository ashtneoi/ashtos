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
        j uart_echo_loop
        li t0, 0x93333
        li t1, 0x100000
        sw t0, 0(t1)

    uart_write_byte:
        li t0, 0x10000000
        sb a0, 0(t0)
        ret

    uart_read_byte_noblock:
        li t0, 0x10000000
        lb t1, 5(t0)
        andi t2, t1, 0x01
        bne t2, zero, _urbn1
        li a1, 1
        ret
    _urbn1:
        lb a0, 0(t0)
        li a1, 0
        ret

    uart_echo_loop:
        call uart_read_byte_noblock
        bne a1, zero, uart_echo_loop
        call uart_write_byte
        j uart_echo_loop
"#);
