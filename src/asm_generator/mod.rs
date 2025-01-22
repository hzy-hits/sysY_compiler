mod gen_instruct;
mod instruct_generator;
mod register_manager;
mod riscv_asm_generator;

use instruct_generator::RiscvInstructionGenerator;
use register_manager::RiscvRegisterManager;

pub struct AsmGenerator {
    reg_manager: RiscvRegisterManager,
    inst_generator: RiscvInstructionGenerator,
    output: String,
}

impl AsmGenerator {
    pub fn new() -> Self {
        Self {
            reg_manager: RiscvRegisterManager::new(),
            inst_generator: RiscvInstructionGenerator,
            output: String::new(),
        }
    }
}
