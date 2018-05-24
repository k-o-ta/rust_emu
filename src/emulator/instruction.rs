pub trait Instruction {
    fn run_instructions(&mut self);
    fn mov_r32_imm32(&mut self);
    fn short_jump(&mut self);
    fn near_jump(&mut self);
    fn exec_instruction(&mut self);
}
