pub struct ModRM {
    pub mode: u8,
    pub reg: Reg,
    pub rm: u8,
    pub sib: u8,
    pub disp: Disp,
}

#[repr(C)]
pub union Reg {
    pub opecode: u8,
    pub reg_index: u8,
}

#[repr(C)]
pub union Disp {
    pub disp8: i8,
    pub disp32: u32,
}

impl ModRM {
    pub fn new() -> ModRM {
        ModRM {
            mode: 0,
            reg: Reg { opecode: 0 },
            rm: 0,
            sib: 0,
            disp: Disp { disp32: 0 },
        }
    }
    pub fn get_opecode(&self) -> u8 {
        unsafe {
            match self.reg {
                Reg { opecode } => opecode,
            }
        }
    }
    pub fn get_reg_index(&self) -> u8 {
        unsafe {
            match self.reg {
                Reg { reg_index } => reg_index,
            }
        }
    }
    pub fn get_disp8(&self) -> i8 {
        unsafe {
            match self.disp {
                Disp { disp8 } => disp8,
            }
        }
    }

    pub fn get_disp32(&self) -> u32 {
        unsafe {
            match self.disp {
                Disp { disp32 } => disp32,
            }
        }
    }
}

pub trait Function {
    fn parse_modrm(&mut self) -> ModRM;
    fn calc_memory_address(&self, &ModRM) -> u32;
    fn get_r32(&mut self, modrm: &ModRM) -> u32;
    fn get_rm32(&mut self, modrm: &ModRM) -> u32;
    fn set_r32(&mut self, modrm: &ModRM, value: u32);
    fn set_rm32(&mut self, modrm: &ModRM, value: u32);
}
