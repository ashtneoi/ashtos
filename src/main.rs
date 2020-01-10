#![no_std]
#![no_builtins]
#![no_main]
#![feature(global_asm)]

use core::panic::PanicInfo;
use core::ptr::{read_volatile, write_volatile};

mod constants;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        // do nothing
    }
}

struct Uart(*mut u8);

impl Uart {
    fn try_write_byte(&self, x: u8) -> bool {
        let base = self.0;
        unsafe {
            let line_status_register = read_volatile(base.wrapping_offset(5));
            let ready = line_status_register & (1<<5) != 0; // THRE
            if ready {
                write_volatile(base, x);
            }
            return ready;
        }
    }

    fn write_byte(&self, x: u8) {
        while !self.try_write_byte(x) { /* spin */ }
    }

    fn try_read_byte(&self) -> Option<u8> {
        let base = self.0;
        unsafe {
            let line_status_register = read_volatile(base.wrapping_offset(5));
            let ready = line_status_register & (1<<0) != 0; // DR
            if ready {
                return Some(read_volatile(base));
            } else {
                return None;
            }
        }
    }

    fn read_byte(&self) -> u8 {
        loop {
            let mx = self.try_read_byte();
            if let Some(x) = mx {
                return x;
            }
        }
    }
}

#[no_mangle]
extern "C" fn rust_go() -> ! {
    let u = Uart(constants::UART0_BASE as *mut u8);
    loop {
        let x = u.read_byte();
        u.write_byte(x);
    }
}

global_asm!(include_str!("go.asm"));
