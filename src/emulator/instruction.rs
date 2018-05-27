use emulator::modrm::ModRM;

pub trait Instruction {
    fn run_instructions(&mut self);
    fn exec_instruction(&mut self);

    fn mov_r32_imm32(&mut self);
    fn move_rm32_imm32(&mut self);
    fn mov_rm32_r32(&mut self);
    fn mov_r32_rm32(&mut self);
    fn add_rm32_r32(&mut self);
    fn sub_rm32_imm8(&mut self, &ModRM);
    fn code_83(&mut self);
    fn inc_rm32(&mut self, &ModRM);
    fn code_ff(&mut self);
    fn short_jump(&mut self);
    fn near_jump(&mut self);
}
