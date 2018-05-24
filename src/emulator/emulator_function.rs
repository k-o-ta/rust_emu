pub trait EmulatorFunction {
    fn get_code8(&self, index: i32) -> u8;
    fn get_sign_code8(&self, index: i32) -> i8;
    fn get_code32(&self, index: i32) -> u32;
    fn get_sign_code32(&self, index: i32) -> i32;
}
