// TODO: Make this build script self-hostable under ashtOS.

use mint_template_engine;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;

fn main() {
    // TODO: Don't use `Result::unwrap()`.

    let mut environ = HashMap::new();
    environ.insert("memory_base", "0x80000000");
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
}
