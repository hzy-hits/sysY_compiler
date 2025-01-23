use koopa::ir::BinaryOp;

use crate::traits::instruct_generator::InstructionGenerator;

pub struct RiscvInstructionGenerator;

impl InstructionGenerator for RiscvInstructionGenerator {
    fn generate_binary(&mut self, op: BinaryOp, dst: &str, lhs: &str, rhs: &str) -> String {
        match op {
            BinaryOp::Eq => format!("  xor {0}, {1}, {2}\n  seqz {0}, {0}\n", dst, lhs, rhs),
            BinaryOp::Sub => format!("  sub {}, {}, {}\n", dst, lhs, rhs),
            BinaryOp::Add => format!("  add {}, {}, {}\n", dst, lhs, rhs),
            BinaryOp::Mul => format!("  mul {}, {}, {}\n", dst, lhs, rhs),
            BinaryOp::Div => format!("  div {}, {}, {}\n", dst, lhs, rhs),
            BinaryOp::Mod => format!("  rem {}, {}, {}\n", dst, lhs, rhs),
            BinaryOp::And => format!("  and {}, {}, {}\n", dst, lhs, rhs),
            BinaryOp::Or => format!("  or {}, {}, {}\n", dst, lhs, rhs),
            BinaryOp::Lt => format!("  slt {}, {}, {}\n", dst, lhs, rhs),
            BinaryOp::Gt => format!("  slt {}, {}, {}\n", dst, rhs, lhs),
            BinaryOp::Le => format!("  slt {0}, {2}, {1}\n  seqz {0}, {0}\n", dst, lhs, rhs),
            BinaryOp::Ge => format!("  slt {0}, {1}, {2}\n  seqz {0}, {0}\n", dst, lhs, rhs),
            BinaryOp::NotEq => format!("  xor {0}, {1}, {2}\n  snez {0}, {0}\n", dst, lhs, rhs),

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
