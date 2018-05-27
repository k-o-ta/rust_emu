pub trait EmulatorFunction {
    fn get_code8(&self, index: i32) -> u8;
    fn get_sign_code8(&self, index: i32) -> i8;
    fn get_code32(&self, index: i32) -> u32;
    fn get_sign_code32(&self, index: i32) -> i32;
    fn get_memory8(&mut self, address: usize) -> u32;
    fn get_memory32(&mut self, address: usize) -> u32;
    fn set_memory8(&mut self, address: usize, value: u32);
    fn set_memory32(&mut self, address: usize, value: u32);
    fn get_register32(&self, usize) -> u32;
    fn set_register32(&mut self, index: usize, value: u32);
}
