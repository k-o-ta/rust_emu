mod emulator_function;
pub mod instruction;
pub mod modrm;

use std::fs::File;
use std::io::{BufReader, Read};

use self::emulator_function::EmulatorFunction;
use self::instruction::Instruction;
use self::modrm::{Disp, Function as ModRMFunction, ModRM};

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

    fn get_memory8(&mut self, address: usize) -> u32 {
        self.memory[address] as u32
    }

    fn get_memory32(&mut self, address: usize) -> u32 {
        let mut ret = 0;
        for i in 0..=3 {
            ret |= (self.get_memory8(address + i)) << (8 * i);
        }
        return ret;
    }

    fn set_memory8(&mut self, address: usize, value: u32) {
        self.memory[address] = (value & 0xFF) as u8;
    }

    fn set_memory32(&mut self, address: usize, value: u32) {
        for i in 0..=3 {
            self.set_memory8(address + i, value >> (i * 8));
        }
    }

    fn get_register32(&self, index: usize) -> u32 {
        self.registers[index]
    }

    fn set_register32(&mut self, index: usize, value: u32) {
        self.registers[index] = value;
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
            0x01 => self.add_rm32_r32(),
            0x83 => self.code_83(),
            0x89 => self.mov_rm32_r32(),
            0x8B => self.mov_r32_rm32(),
            0xB8...0xBF => self.mov_r32_imm32(),
            0xC7 => self.move_rm32_imm32(),
            0xE9 => self.near_jump(),
            0xEB => self.short_jump(),
            0xFF => self.code_ff(),
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

    fn move_rm32_imm32(&mut self) {
        self.eip += 1;
        let modrm = self.parse_modrm();
        let value = self.get_code32(0);
        self.eip += 4;
        self.set_rm32(&modrm, value);
    }

    fn mov_rm32_r32(&mut self) {
        self.eip += 1;
        let modrm = self.parse_modrm();
        let r32 = self.get_r32(&modrm);
        self.set_rm32(&modrm, r32);
    }

    fn mov_r32_rm32(&mut self) {
        self.eip += 1;
        let modrm = self.parse_modrm();
        let rm32 = self.get_rm32(&modrm);
        self.set_r32(&modrm, rm32);
    }

    fn add_rm32_r32(&mut self) {
        self.eip += 1;
        let modrm = self.parse_modrm();
        let r32 = self.get_r32(&modrm);
        let rm32 = self.get_rm32(&modrm);
        self.set_rm32(&modrm, rm32 + r32);
    }

    fn sub_rm32_imm8(&mut self, modrm: &ModRM) {
        let rm32 = self.get_rm32(&modrm);
        let imm8 = self.get_sign_code8(0) as i32;
        self.eip += 1;
        // imm8の正負で条件分岐
        if imm8 >= 0 {
            self.set_rm32(&modrm, rm32 - (imm8 as u32));
        } else {
            self.set_rm32(&modrm, rm32 + (imm8.abs() as u32));
        }
    }

    fn code_83(&mut self) {
        self.eip += 1;
        let modrm = self.parse_modrm();
        match modrm.get_opecode() {
            5 => self.sub_rm32_imm8(&modrm),
            opecode @ _ => panic!("not implemented: 83 /{}", opecode),
        }
    }
    fn inc_rm32(&mut self, modrm: &ModRM) {
        let value = self.get_rm32(&modrm);
        self.set_rm32(&modrm, value + 1);
    }

    fn code_ff(&mut self) {
        self.eip += 1;
        let modrm = self.parse_modrm();
        match (modrm.get_opecode()) {
            0 => self.inc_rm32(&modrm),
            opecode @ _ => panic!("not implemented: FF /{}", opecode),
        }
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
impl ModRMFunction for Emulator {
    fn parse_modrm(&mut self) -> ModRM {
        let mut modrm = ModRM::new();

        let code = self.get_code8(0);
        modrm.mode = (code & 0xC0) >> 6;
        modrm.reg.opecode = (code & 0x38) >> 3;
        modrm.rm = code & 0x07;

        self.eip += 1;

        if modrm.mode != 3 && modrm.rm == 4 {
            modrm.sib = code;
            self.eip += 1;
        }

        if (modrm.mode == 0 && modrm.rm == 5) || modrm.mode == 2 {
            modrm.disp.disp32 = self.get_sign_code32(0) as u32;
            self.eip += 4;
        } else if (modrm.mode == 1) {
            modrm.disp.disp8 = self.get_sign_code8(0);
            self.eip += 1;
        }
        modrm
    }

    fn calc_memory_address(&self, modrm: &ModRM) -> u32 {
        match modrm.mode {
            0 => match modrm.rm {
                4 => panic!("not implemented ModRM mode = 0, rm = 4"),
                5 => modrm.get_disp32(),
                _ => self.get_register32(modrm.rm as usize),
            },
            1 => match modrm.rm {
                4 => panic!("not implemented ModRM mode = 1, rm = 4"),
                _ => self.get_register32(modrm.rm as usize) + modrm.get_disp8() as u32,
            },
            2 => match modrm.rm {
                4 => panic!("not implemented ModRM mode = 2, rm = 4"),
                _ => self.get_register32(modrm.rm as usize) + modrm.get_disp32() as u32,
            },
            _ => {
                panic!("not implemented ModRM mod = 3");
            }
        }
    }

    fn get_r32(&mut self, modrm: &ModRM) -> u32 {
        self.get_register32(modrm.get_reg_index() as usize)
    }

    fn get_rm32(&mut self, modrm: &ModRM) -> u32 {
        if modrm.mode == 3 {
            self.get_register32(modrm.rm as usize)
        } else {
            let address = self.calc_memory_address(modrm);
            self.get_memory32(address as usize)
        }
    }

    fn set_r32(&mut self, modrm: &ModRM, value: u32) {
        self.set_register32(modrm.get_reg_index() as usize, value);
    }

    fn set_rm32(&mut self, modrm: &ModRM, value: u32) {
        if modrm.mode == 3 {
            self.set_register32(modrm.rm as usize, value);
        } else {
            let address = self.calc_memory_address(modrm);
            self.set_memory32(address as usize, value);
        }
    }
}
