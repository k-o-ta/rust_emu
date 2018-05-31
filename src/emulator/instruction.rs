use emulator::modrm::ModRM;

pub trait Instruction {
    fn run_instructions(&mut self);
    fn exec_instruction(&mut self);

    fn mov_r32_imm32(&mut self);
    fn move_rm32_imm32(&mut self);
    fn mov_rm32_r32(&mut self);
    fn mov_r32_rm32(&mut self);
    fn add_rm32_r32(&mut self);
    fn add_rm32_imm8(&mut self, &ModRM);
    fn sub_rm32_imm8(&mut self, &ModRM);
    fn cmp_rm32_imm8(&mut self, &ModRM);
    fn cmp_r32_rm32(&mut self);
    fn code_83(&mut self);
    fn inc_rm32(&mut self, &ModRM);
    fn code_ff(&mut self);
    fn push_r32(&mut self);
    fn pop_r32(&mut self);
    fn call_rel32(&mut self);
    fn ret(&mut self);
    fn leave(&mut self);
    fn push_imm8(&mut self);
    fn push_imm32(&mut self);
    fn short_jump(&mut self);
    fn near_jump(&mut self);
    fn jo(&mut self);
    fn jno(&mut self);
    fn jc(&mut self);
    fn jnc(&mut self);
    fn jz(&mut self);
    fn jnz(&mut self);
    fn js(&mut self);
    fn jns(&mut self);
    fn jl(&mut self);
    fn jle(&mut self);
}
