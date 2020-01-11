#![no_std]
#![no_builtins]
#![no_main]
#![feature(global_asm)]

use core::panic::PanicInfo;
use core::ptr::{read_volatile, write_volatile};

use riscv::register::{self, misa::MXL};

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

    fn write_bytes(&self, x: &[u8]) {
        for byte in x {
            self.write_byte(*byte);
        }
    }

    /// provisional
    fn write_nibble_hex(&self, x: u8) {
        static HEX_DIGITS: &[u8] = &[
            0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39,
            0x41, 0x42, 0x43, 0x44, 0x45, 0x46
        ];
        self.write_byte(HEX_DIGITS[x as usize]);
    }

    /// provisional
    fn write_byte_hex(&self, x: u8) {
        self.write_nibble_hex(x & 0xF);
        self.write_nibble_hex(x >> 4);
    }

    /// provisional
    fn write_int_hex(&self, mut x: usize, digit_count: usize) {
        for i in (0..digit_count).rev() {
            self.write_nibble_hex((((0xF << (4 * i)) & x) >> (4 * i)) as u8);
            if i % 4 == 0 && i != 0 {
                self.write_byte('_' as u8);
            }
        }
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
    if register::mhartid::read() != 0 {
        loop { }
    }

    let u = Uart(constants::UART0_BASE as *mut u8);

    u.write_bytes(b"<<<=== ashtOS-fw ===>>>\n");
    u.write_byte('\n' as u8);

    u.write_bytes(b"machine info:\n");

    // misa //

    let maybe_misa = register::misa::read();

    u.write_bytes(b"  misa:      0x");
    let misa_bits = match maybe_misa {
        Some(misa) => misa.bits(),
        None => 0,
    };
    u.write_int_hex(misa_bits, 16);
    u.write_byte('\n' as u8);

    if let Some(misa) = maybe_misa {
        u.write_bytes(b"    MXLEN:   ");
        u.write_bytes(match misa.mxl() {
            MXL::XLEN32 => b"32",
            MXL::XLEN64 => b"64",
            MXL::XLEN128 => b"128",
        });
        u.write_byte('\n' as u8);

        let mut misa_bits = misa_bits;
        if misa_bits & (1<<6) != 0 {
            u.write_bytes(b"    note:    extension bit \"G\" is set\n");
            u.write_bytes(b"             but I don't know how to decode\n");
            u.write_bytes(b"             additional standard extensions\n");
            misa_bits &= !(1<<6);
        }
        static G_BITS: usize = (1<<8) | (1<<12) | (1<<0) | (1<<5) | (1<<3);
        if misa_bits & G_BITS != 0 {
            misa_bits &= !G_BITS;
            misa_bits |= 1<<6;
        }
        u.write_bytes(b"    exts:    ");
        for i in 0..=25 {
            if misa_bits & 1 != 0 {
                u.write_byte('A' as u8 + i as u8);
            }
            misa_bits >>= 1;
        }
    }
    u.write_byte('\n' as u8);

    // mvendorid //

    u.write_bytes(b"  mvendorid: 0x");
    let mvendorid = match register::mvendorid::read() {
        Some(x) => x.bits(),
        None => 0,
    };
    u.write_int_hex(mvendorid, 16);
    u.write_byte('\n' as u8);

    // marchid //

    u.write_bytes(b"  marchid:   0x");
    let marchid = match register::marchid::read() {
        Some(x) => x.bits(),
        None => 0,
    };
    u.write_int_hex(marchid, 16);
    u.write_byte('\n' as u8);

    // mimpid //

    u.write_bytes(b"  mimpid:    0x");
    let mimpid = match register::mimpid::read() {
        Some(x) => x.bits(),
        None => 0,
    };
    u.write_int_hex(mimpid, 16);
    u.write_byte('\n' as u8);

    // mcause //

    u.write_bytes(b"  mcause:    0x");
    let mcause = register::mcause::read().bits();
    u.write_int_hex(mcause, 16);
    u.write_byte('\n' as u8);

    // ... //

    u.write_byte('\n' as u8);

    loop {
        let x = u.read_byte();
        u.write_byte(x);
    }
}

#[no_mangle]
extern "C" fn abort() -> ! {
    let u = Uart(constants::UART0_BASE as *mut u8);
    u.write_bytes(b"<<<ashtOS abort>>>\n");
    loop { }
}

global_asm!(include_str!("go.asm"));
