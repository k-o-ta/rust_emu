pub trait EmulatorFunction {
    const CARRY_FLAG: u32;
    const ZERO_FLAG: u32;
    const SIGN_FLAG: u32;
    const OVERFLOW_FLAG: u32;
    fn get_code8(&self, index: i32) -> u8;
    fn get_sign_code8(&self, index: i32) -> i8;
    fn get_code32(&self, index: i32) -> u32;
    fn get_sign_code32(&self, index: i32) -> i32;
    fn get_memory8(&mut self, address: usize) -> u32;
    fn get_memory32(&mut self, address: usize) -> u32;
    fn set_memory8(&mut self, address: usize, value: u32);
    fn set_memory32(&mut self, address: usize, value: u32);
    fn get_register8(&self, usize) -> u8;
    fn get_register32(&self, usize) -> u32;
    fn set_register8(&mut self, index: usize, value: u8);
    fn set_register32(&mut self, index: usize, value: u32);
    fn push32(&mut self, u32);
    fn pop32(&mut self) -> u32;
    fn is_carry(&self) -> bool;
    fn is_zero(&self) -> bool;
    fn is_sign(&self) -> bool;
    fn is_overflow(&self) -> bool;
    fn set_carry(&mut self, bool);
    fn set_sign(&mut self, bool);
    fn set_zero(&mut self, bool);
    fn set_overflow(&mut self, bool);
    fn update_eflags_sub(&mut self, v1: u32, v2: u32, result: u64);
}
