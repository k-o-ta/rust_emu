extern crate rust_emu;

use rust_emu::emulator::Emulator;
use rust_emu::emulator::instruction::Instruction;

fn main() {
    let mut args: Vec<String> = std::env::args().collect();
    args.dedup();

    let quiet = args.iter().find(|&arg| *arg == "-q".to_string()).is_some();
    args.retain(|ref arg| **arg != "-q".to_string());

    if args.len() != 2 {
        eprintln!("usage: px86 filename");
        ::std::process::exit(1);
    }

    let mut emu: Emulator;
    if let Ok(f) = ::std::fs::File::open(&args[1]) {
        emu = Emulator::new(rust_emu::emulator::MEMORY_SIZE, 0x7c00, 0x7c00, f);
    } else {
        eprintln!("ファイルが開けません: {}", &args[1]);
        ::std::process::exit(1);
    }
    emu.run_instructions(quiet);
    emu.dump_registers();
}
