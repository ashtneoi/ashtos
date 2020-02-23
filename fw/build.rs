// TODO: Make this build script self-hostable under ashtOS.

// FIXME: This isn't supposed to modify files except under OUT_DIR.

use mint_template_engine;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::process::Command;

fn main() {
    // TODO: Don't use `Result::unwrap()`.

    let mut environ = HashMap::new();
    environ.insert("mem_base", "0x80000000");
    environ.insert("mem_length", "128M");
    environ.insert("reset_vector", "0x80000000");
    environ.insert("vector_table_base", "0x80000100");
    environ.insert("stack_base", "0x80010000");
    environ.insert("allocation_base", "0x80010000");
    environ.insert("allocation_cap", "0x10000");
    environ.insert("uart0_base", "0x10000000");

    // TODO: Refactor.
    {
        let lines = mint_template_engine::do_file(
            "link.ld.tmpl", &environ
        ).unwrap();
        let mut f = File::create("link.ld").unwrap();
        for line in lines {
            f.write(line.as_bytes()).unwrap();
            f.write(b"\n").unwrap();
        }
    }
    {
        let lines = mint_template_engine::do_file(
            "src/constants.rs.tmpl", &environ
        ).unwrap();
        let mut f = File::create("src/constants.rs").unwrap();
        for line in lines {
            f.write(line.as_bytes()).unwrap();
            f.write(b"\n").unwrap();
        }
    }
    {
        let lines = mint_template_engine::do_file(
            "src/go.asm.tmpl", &environ
        ).unwrap();
        let mut f = File::create("src/go.asm").unwrap();
        for line in lines {
            f.write(line.as_bytes()).unwrap();
            f.write(b"\n").unwrap();
        }
    }

    let rustc_path = env::var("RUSTC").unwrap();
    let version_line =
        Command::new(rustc_path).arg("--version").output().unwrap().stdout;
    println!("{}", String::from_utf8(version_line).unwrap());
}
