mod emulator_function;
pub mod instruction;

use std::fs::File;
use std::io::{BufReader, Read};

use emulator::emulator_function::EmulatorFunction;
use emulator::instruction::Instruction;

// メモリは1MB
pub const MEMORY_SIZE: usize = 1024 * 1024;
const REGISTERS_NAME: [&str; 8] = ["EAX", "ECX", "EDX", "EBX", "ESP", "EBP", "ESI", "EDI"];

enum Register {
    EAX,
    ECX,
    EDX,
    EBX,
    ESP,
    EBP,
    ESI,
    EDI,
    RegistersCount,
}

pub struct Emulator {
    // 汎用レジスタ
    registers: [u32; Register::RegistersCount as usize],
    // EFLAGSレジスタ
    eflags: u32,
    // メモリ(バイト列)
    pub memory: Vec<u8>,
    // プログラムカウンタ
    eip: u32,
}

impl EmulatorFunction for Emulator {
    fn get_code8(&self, index: i32) -> u8 {
        self.memory[(self.eip + index as u32) as usize]
    }

    fn get_sign_code8(&self, index: i32) -> i8 {
        self.memory[(self.eip + index as u32) as usize] as i8
    }

    fn get_code32(&self, index: i32) -> u32 {
        let mut ret: u32 = 0;

        // リトルエンディアンでメモリの値を取得する
        for i in 0..=3 {
            ret |= (self.get_code8(index + i) as u32) << (i * 8);
        }
        return ret;
    }

    fn get_sign_code32(&self, index: i32) -> i32 {
        self.get_code32(index) as i32
    }
}

impl Instruction for Emulator {
    fn run_instructions(&mut self) {
        while self.eip < MEMORY_SIZE as u32 {
            self.exec_instruction();
            if self.eip == 0x00 {
                println!("end of program.");
                break;
            }
        }
    }

    fn exec_instruction(&mut self) {
        let code = self.get_code8(0);
        // 現在のプログラムカウンタと実行されるバイナリを出力する
        println!("EIP = {:0X}, Code = {:02X}", self.eip, code);
        match code {
            0xB8...0xBF => self.mov_r32_imm32(),
            0xE9 => self.near_jump(),
            0xEB => self.short_jump(),
            _ => {
                eprintln!("Not Implemented: {:0x}", code);
                ::std::process::exit(1);
            }
        }
    }

    fn mov_r32_imm32(&mut self) {
        let reg = self.get_code8(0) - 0xB8;
        let value = self.get_code32(1);
        self.registers[reg as usize] = value;
        self.eip += 5;
    }

    fn short_jump(&mut self) {
        let diff: i8 = self.get_sign_code8(1) + 2;
        match diff > 0 {
            true => {
                self.eip += diff as u32;
            }
            false => {
                self.eip -= diff.abs() as u32;
            }
        }
    }

    fn near_jump(&mut self) {
        let diff = self.get_sign_code32(1) + 5;
        match diff > 0 {
            true => {
                self.eip += diff as u32;
            }
            false => {
                self.eip -= diff.abs() as u32;
            }
        }
    }
}

impl Emulator {
    pub fn new(size: usize, eip: u32, esp: u32, file: File) -> Emulator {
        let memory: Vec<u8> = vec![0; size];
        let mut registers = [0; Register::RegistersCount as usize];
        registers[Register::ESP as usize] = esp;
        let mut emu = Emulator {
            registers: registers,
            eflags: 0,
            memory: memory,
            eip: eip,
        };
        let mut br = BufReader::new(file);
        let _ = br.read_exact(&mut emu.memory[0x7c00..(0x7c00 + 0x201)]);
        emu
    }

    pub fn dump_registers(&self) {
        for i in 0..Register::RegistersCount as usize {
            println!("{} = {:08x}", REGISTERS_NAME[i], self.registers[i]);
        }
        println!("EIP = {:08x}", self.eip);
    }
}
