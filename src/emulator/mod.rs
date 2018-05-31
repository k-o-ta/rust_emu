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
    const CARRY_FLAG: u32 = 1;
    const ZERO_FLAG: u32 = (1 << 6);
    const SIGN_FLAG: u32 = (1 << 7);
    const OVERFLOW_FLAG: u32 = (1 << 11);
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

    fn push32(&mut self, value: u32) {
        let address = self.get_register32(Register::ESP as usize) - 4;
        self.set_register32(Register::ESP as usize, address);
        self.set_memory32(address as usize, value);
    }

    fn pop32(&mut self) -> u32 {
        let address = self.get_register32(Register::ESP as usize);
        let ret = self.get_memory32(address as usize);
        self.set_register32(Register::ESP as usize, address + 4);
        ret
    }

    fn is_carry(&self) -> bool {
        (self.eflags & Self::CARRY_FLAG) != 0
    }
    fn is_zero(&self) -> bool {
        (self.eflags & Self::ZERO_FLAG) != 0
    }

    fn is_sign(&self) -> bool {
        (self.eflags & Self::SIGN_FLAG) != 0
    }

    fn is_overflow(&self) -> bool {
        (self.eflags & Self::OVERFLOW_FLAG) != 0
    }

    fn set_carry(&mut self, is_carry: bool) {
        if (is_carry) {
            self.eflags |= Self::CARRY_FLAG;
        } else {
            self.eflags &= !Self::CARRY_FLAG;
        }
    }

    fn set_sign(&mut self, is_sign: bool) {
        if (is_sign) {
            self.eflags |= Self::SIGN_FLAG;
        } else {
            self.eflags &= !Self::SIGN_FLAG;
        }
    }

    fn set_zero(&mut self, is_zero: bool) {
        if (is_zero) {
            self.eflags |= Self::ZERO_FLAG;
        } else {
            self.eflags &= !Self::ZERO_FLAG;
        }
    }

    fn set_overflow(&mut self, is_overflow: bool) {
        if (is_overflow) {
            self.eflags |= Self::OVERFLOW_FLAG;
        } else {
            self.eflags &= !Self::OVERFLOW_FLAG;
        }
    }

    fn update_eflags_sub(&mut self, v1: u32, v2: u32, result: u64) {
        let sign1 = (v1 >> 31) == 1;
        let sign2 = (v2 >> 31) == 1;
        let signr = ((result >> 31) & 1) == 1;

        self.set_carry((result >> 32) != 0);
        self.set_zero(result == 0);
        self.set_sign(signr);
        self.set_overflow(sign1 != sign2 && sign1 != signr);
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
            0x3B => self.cmp_r32_rm32(),
            0x50..=0x57 => self.push_r32(),
            0x58..=0x5f => self.pop_r32(),
            0x68 => self.push_imm32(),
            0x6A => self.push_imm8(),
            0x70 => self.jo(),
            0x71 => self.jno(),
            0x72 => self.jc(),
            0x73 => self.jnc(),
            0x74 => self.jz(),
            0x75 => self.jnz(),
            0x78 => self.js(),
            0x79 => self.jns(),
            0x7C => self.jl(),
            0x7E => self.jle(),
            0x83 => self.code_83(),
            0x89 => self.mov_rm32_r32(),
            0x8B => self.mov_r32_rm32(),
            0xB8..=0xBF => self.mov_r32_imm32(),
            0xC3 => self.ret(),
            0xC7 => self.move_rm32_imm32(),
            0xC9 => self.leave(),
            0xE8 => self.call_rel32(),
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

    fn add_rm32_imm8(&mut self, modrm: &ModRM) {
        let rm32 = self.get_rm32(modrm);
        let imm8 = self.get_sign_code8(0) as i32;
        self.eip += 1;

        let sum: u32;
        if imm8 > 0 {
            sum = rm32 + imm8 as u32;
        } else {
            sum = rm32 - (imm8.abs() as u32);
        }
        self.set_rm32(modrm, sum);
    }

    fn sub_rm32_imm8(&mut self, modrm: &ModRM) {
        let rm32 = self.get_rm32(&modrm);
        let imm8 = self.get_sign_code8(0) as i32;
        self.eip += 1;
        // imm8の正負で条件分岐
        let result = if imm8 >= 0 { rm32 - (imm8 as u32) } else { rm32 + (imm8.abs() as u32) };
        self.set_rm32(&modrm, result);
        self.update_eflags_sub(rm32, imm8 as u32, result as u64);
    }

    fn cmp_rm32_imm8(&mut self, modrm: &ModRM) {
        let rm32 = self.get_rm32(modrm);
        let imm8 = self.get_sign_code8(0) as i32;
        self.eip += 1;
        let result = if imm8 >= 0 { rm32 - (imm8 as u32) } else { rm32 + (imm8.abs() as u32) };
        self.update_eflags_sub(rm32, imm8 as u32, result as u64);
    }

    fn cmp_r32_rm32(&mut self) {
        self.eip += 1;
        let modrm = self.parse_modrm();
        let r32 = self.get_r32(&modrm);
        let rm32 = self.get_rm32(&modrm);
        let result: u64 = r32 as u64 - rm32 as u64;
        self.update_eflags_sub(r32, rm32, result);
    }

    fn code_83(&mut self) {
        self.eip += 1;
        let modrm = self.parse_modrm();
        match modrm.get_opecode() {
            0 => self.add_rm32_imm8(&modrm),
            5 => self.sub_rm32_imm8(&modrm),
            7 => self.cmp_rm32_imm8(&modrm),
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
        match modrm.get_opecode() {
            0 => self.inc_rm32(&modrm),
            opecode @ _ => panic!("not implemented: FF /{}", opecode),
        }
    }

    fn push_r32(&mut self) {
        let push_r32_code = 0x50;
        let reg = self.get_code8(0) - push_r32_code;
        let value = self.get_register32(reg as usize);
        self.push32(value);
        self.eip += 1;
    }

    fn push_imm8(&mut self) {
        let value = self.get_code8(1);
        self.push32(value as u32);
        self.eip += 2;
    }

    fn push_imm32(&mut self) {
        let value = self.get_code32(1);
        self.push32(value);
        self.eip += 5;
    }

    fn pop_r32(&mut self) {
        let pop_r32_code = 0x58;
        let reg = self.get_code8(0) - pop_r32_code;
        let value = self.pop32();
        self.set_register32(reg as usize, value);
        self.eip += 1;
    }

    fn call_rel32(&mut self) {
        let diff = self.get_sign_code32(1);
        let eip = self.eip;
        self.push32(eip + 5);
        if diff + 5 > 0 {
            self.eip += (diff + 5) as u32;
        } else {
            self.eip -= (diff + 5).abs() as u32;
        }
    }

    fn ret(&mut self) {
        self.eip = self.pop32();
    }

    fn leave(&mut self) {
        let ebp = self.get_register32(Register::EBP as usize);
        self.set_register32(Register::ESP as usize, ebp);
        let value = self.pop32();
        self.set_register32(Register::EBP as usize, value);
        self.eip += 1;
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

    fn jo(&mut self) {
        let diff = if self.is_overflow() { self.get_sign_code8(1) } else { 0 };
        if diff + 2 > 0 {
            self.eip += (diff + 2) as u32;
        } else {
            self.eip -= (diff + 2).abs() as u32;
        }
    }

    fn jno(&mut self) {
        let diff = if self.is_overflow() { 0 } else { self.get_sign_code8(1) };
        if diff + 2 > 0 {
            self.eip += (diff + 2) as u32;
        } else {
            self.eip -= (diff + 2).abs() as u32;
        }
    }

    fn jc(&mut self) {
        let diff = if self.is_carry() { self.get_sign_code8(1) } else { 0 };
        if diff + 2 > 0 {
            self.eip += (diff + 2) as u32;
        } else {
            self.eip -= (diff + 2).abs() as u32;
        }
    }

    fn jnc(&mut self) {
        let diff = if self.is_carry() { 0 } else { self.get_sign_code8(1) };
        if diff + 2 > 0 {
            self.eip += (diff + 2) as u32;
        } else {
            self.eip -= (diff + 2).abs() as u32;
        }
    }

    fn jz(&mut self) {
        let diff = if self.is_zero() { self.get_sign_code8(1) } else { 0 };
        if diff + 2 > 0 {
            self.eip += (diff + 2) as u32;
        } else {
            self.eip -= (diff + 2).abs() as u32;
        }
    }

    fn jnz(&mut self) {
        let diff = if self.is_zero() { 0 } else { self.get_sign_code8(1) };
        if diff + 2 > 0 {
            self.eip += (diff + 2) as u32;
        } else {
            self.eip -= (diff + 2).abs() as u32;
        }
    }

    fn js(&mut self) {
        let diff = if self.is_sign() { self.get_sign_code8(1) } else { 0 };
        if diff + 2 > 0 {
            self.eip += (diff + 2) as u32;
        } else {
            self.eip -= (diff + 2).abs() as u32;
        }
    }

    fn jns(&mut self) {
        let diff = if self.is_sign() { 0 } else { self.get_sign_code8(1) };
        if diff + 2 > 0 {
            self.eip += (diff + 2) as u32;
        } else {
            self.eip -= (diff + 2).abs() as u32;
        }
    }

    fn jl(&mut self) {
        let diff = if self.is_sign() != self.is_overflow() {
            self.get_sign_code8(1)
        } else {
            0
        };
        if diff + 2 > 0 {
            self.eip += (diff + 2) as u32;
        } else {
            self.eip -= (diff + 2).abs() as u32;
        }
    }

    fn jle(&mut self) {
        let diff = if self.is_zero() || (self.is_sign() != self.is_overflow()) {
            self.get_sign_code8(1)
        } else {
            0
        };
        if diff + 2 > 0 {
            self.eip += (diff + 2) as u32;
        } else {
            self.eip -= (diff + 2).abs() as u32;
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
        } else if modrm.mode == 1 {
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
                _ => {
                    let disp = modrm.get_disp8();
                    if disp > 0 {
                        self.get_register32(modrm.rm as usize) + modrm.get_disp8() as u32
                    } else {
                        self.get_register32(modrm.rm as usize) - modrm.get_disp8().abs() as u32
                    }
                }
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
