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
    .global uart_write_byte_noblock

    .section .text

    .org 0x0
    go:
        li sp, 0x80004000

        la a0, boot_splash_str
        jal uart_write_bytes

        j virt_exit
        j uart_echo_loop

    uart_write_bytes:
        addi sp, sp, -8
        sd ra, (sp)
        addi sp, sp, -8
        sd s0, (sp)
        mv s0, a0
    0:
        lb a0, (s0)
        beq a0, zero, 2f
    1:
        jal uart_write_byte_noblock
        bne a0, zero, 0b
        addi s0, s0, 1
        j 0b
    2:
        ld s0, (sp)
        addi sp, sp, 8
        ld ra, (sp)
        addi sp, sp, 8
        ret

    uart_write_byte_noblock:
        li t0, 0x10000000
        lb t1, 5(t0)
        andi t2, t1, 1<<5  # THRE
        bne t2, zero, 0f
        li a0, 1
        ret
    0:
        sb a0, 0(t0)
        li a0, 0
        ret

    uart_read_byte_noblock:
        li t0, 0x10000000
        lb t1, 5(t0)
        andi t2, t1, 1<<0  # DR
        bne t2, zero, 0f
        li a1, 1
        ret
    0:
        lb a0, 0(t0)
        li a1, 0
        ret

    uart_echo_loop:
        jal uart_read_byte_noblock
        bne a1, zero, uart_echo_loop
        mv s0, a0
    0:
        mv a0, s0
        jal uart_write_byte_noblock
        bne a0, zero, 0b
        j uart_echo_loop

    virt_exit:
        li t0, 0x93333
        li t1, 0x100000
        sw t0, (t1)

    end_of_text:
        .int 0, 0, 0, 0

    .section .rodata

    boot_splash_str:
        .asciz "=== ashtOSfw ===\r\n"
"#);
