#![no_std]
#![no_builtins]
#![no_main]
#![feature(global_asm)]

use core::fmt::{self, Write};
use core::panic::PanicInfo;
use core::ptr::{read_volatile, write_volatile};

use riscv::register::{self, misa::MXL, mtvec::TrapMode};

mod constants;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // TODO: Dump registers and other useful info.
    let u = Uart(constants::UART0_BASE as *mut u8);
    u.write_bytes(b"<<<--- ashtOS-fw panic --->>>\n");
    loop { }
}

pub struct Uart(*mut u8);

impl Uart {
    pub fn try_write_byte(&self, x: u8) -> bool {
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

    pub fn write_byte(&self, x: u8) {
        while !self.try_write_byte(x) { /* spin */ }
    }

    pub fn write_bytes(&self, x: &[u8]) {
        for byte in x {
            self.write_byte(*byte);
        }
    }

    pub fn write(&self, x: &str) {
        self.write_bytes(x.as_bytes());
    }

    /// provisional
    pub fn write_nibble_hex(&self, x: u8) {
        static HEX_DIGITS: &[u8] = &[
            0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39,
            0x41, 0x42, 0x43, 0x44, 0x45, 0x46
        ];
        self.write_byte(HEX_DIGITS[x as usize]);
    }

    /// provisional
    pub fn write_int_hex(&self, x: usize, digit_count: usize) {
        for i in (0..digit_count).rev() {
            self.write_nibble_hex((((0xF << (4 * i)) & x) >> (4 * i)) as u8);
            if i % 4 == 0 && i != 0 {
                self.write_byte('_' as u8);
            }
        }
    }

    pub fn try_read_byte(&self) -> Option<u8> {
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

    pub fn read_byte(&self) -> u8 {
        loop {
            let mx = self.try_read_byte();
            if let Some(x) = mx {
                return x;
            }
        }
    }
}

impl fmt::Write for Uart {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write(s);
        Ok(())
    }
}

#[no_mangle]
fn main() {
    if register::mhartid::read() != 0 {
        loop { }
    }

    let mut u = Uart(constants::UART0_BASE as *mut u8);
    let u = &mut u;

    u.write("<<<=== ashtOS-fw ===>>>\n");
    u.write("\n");

    u.write("machine info:\n");

    // misa //

    let maybe_misa = register::misa::read();

    let misa_bits = match maybe_misa {
        Some(misa) => misa.bits(),
        None => 0,
    };
    write!(u, "  misa:      0x{:016X}\n", misa_bits).unwrap();

    if let Some(misa) = maybe_misa {
        let mxl_str = match misa.mxl() {
            MXL::XLEN32 => "32",
            MXL::XLEN64 => "64",
            MXL::XLEN128 => "128",
        };
        write!(u, "    MXLEN:   {}\n", mxl_str).unwrap();

        static STANDARD_EXT_NAME_ORDER: &[char] = &[
            'I',
            'E',
            'M',
            'A',
            'F',
            'D',
            'Q',
            'L',
            'C',
            'B',
            'J',
            'T',
            'P',
            'V',
            'N',
        ];

        // Standard extensions whose ordering is unspecified.
        static UNSPEC_STANDARD_EXT_NAME_ORDER: &[char] = &[
            // G means additional standard extensions
            'H',
            'K',
            'O',
            'R',
            'S',
            'U',
            'W',
            'Y',
            'Z',
        ];

        u.write("    exts:    ");
        for &ext_name in STANDARD_EXT_NAME_ORDER {
            if misa.has_extension(ext_name) {
                u.write_byte(ext_name as u8);
            }
        }
        let mut has_unspec_standard_exts = false;
        for &ext_name in UNSPEC_STANDARD_EXT_NAME_ORDER {
            if misa.has_extension(ext_name) {
                has_unspec_standard_exts = true;
            }
        }
        if has_unspec_standard_exts {
            u.write(" (+");
            for &ext_name in UNSPEC_STANDARD_EXT_NAME_ORDER {
                if misa.has_extension(ext_name) {
                    u.write_byte(ext_name as u8);
                }
            }
            u.write(")");
        }
        u.write("\n");
        if misa.has_extension('G') {
            u.write("    note:    Extension bit \"G\" is set,\n");
            u.write("             but I don't know how to decode\n");
            u.write("             additional standard extensions.\n");
        }
        if misa.has_extension('X') {
            u.write("    note:    Extension bit \"X\" is set,\n");
            u.write("             but I don't know how to decode\n");
            u.write("             nonstandard extensions.\n");
        }
    }

    // mvendorid //

    let mvendorid = match register::mvendorid::read() {
        Some(x) => x.bits(),
        None => 0,
    };
    write!(u, "  mvendorid: 0x{:016X}\n", mvendorid).unwrap();

    // marchid //

    let marchid = match register::marchid::read() {
        Some(x) => x.bits(),
        None => 0,
    };
    write!(u, "  marchid:   0x{:016X}\n", marchid).unwrap();

    // mimpid //

    let mimpid = match register::mimpid::read() {
        Some(x) => x.bits(),
        None => 0,
    };
    write!(u, "  mimpid:    0x{:016X}\n", mimpid).unwrap();

    // mcause //

    let mcause = register::mcause::read().bits();
    write!(u, "  mcause:    0x{:016X}", mcause).unwrap();

    // ... //

    u.write("\n");

    u.write("Switching to vectored interrupts...");
    let mtvec_addr = constants::VECTOR_TABLE_BASE;
    unsafe {
        register::mtvec::write(mtvec_addr, TrapMode::Vectored);
    }
    let new_mtvec = register::mtvec::read();
    let addr_okay = new_mtvec.address() == mtvec_addr;
    let mode_okay = new_mtvec.trap_mode() == TrapMode::Vectored;
    if addr_okay && mode_okay {
        u.write("done.\n");
    } else {
        u.write("failed!\n");
        if !addr_okay {
            u.write_bytes(b"  - Can't set base address.\n");
            write!(
                u, "    Wanted 0x{:016X}.\n", mtvec_addr
            ).unwrap();
            write!(
                u, "    Got    0x{:016X}.\n", new_mtvec.address()
            ).unwrap();
        }
        if !mode_okay {
            u.write(
                "  - Can't set mode. Wrote 'vectored', read 'direct'.\n"
            );
        }
        abort();
    }

    loop { }
}

#[no_mangle]
extern "C" fn rust_go() -> ! {
    main();

    loop { }
}

#[no_mangle]
extern "C" fn abort() -> ! {
    // TODO: Dump registers and other useful info.
    let u = Uart(constants::UART0_BASE as *mut u8);
    u.write_bytes(b"<<<--- ashtOS-fw abort --->>>\n");
    loop { }
}

global_asm!(include_str!("go.asm"));
