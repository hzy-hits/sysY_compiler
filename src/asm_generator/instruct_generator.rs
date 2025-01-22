use koopa::ir::BinaryOp;

use crate::traits::instruct_generator::InstructionGenerator;

pub struct RiscvInstructionGenerator;

impl InstructionGenerator for RiscvInstructionGenerator {
    fn generate_binary(&mut self, op: BinaryOp, dst: &str, lhs: &str, rhs: &str) -> String {
        match op {
            BinaryOp::Eq => format!("  xor {0}, {1}, {2}\n  seqz {0}, {0}\n", dst, lhs, rhs),
            BinaryOp::Sub => format!("  sub {}, {}, {}\n", dst, lhs, rhs),
            _ => unreachable!("Unsupported binary operation"),
        }
    }

    fn generate_return(&mut self, val_reg: Option<&str>) -> String {
        match val_reg {
            Some(reg) => format!("  mv a0, {}\n  ret\n", reg),
            None => "  ret\n".to_string(),
        }
    }

    fn generate_load_immediate(&mut self, dst: &str, value: i32) -> String {
        format!("  li {}, {}\n", dst, value)
    }

    fn generate_move(&mut self, dst: &str, src: &str) -> String {
        format!("  mv {}, {}\n", dst, src)
    }
}
