use anyhow::Result;
use koopa::ir::BinaryOp;
pub trait InstructionGenerator {
    fn generate_binary(&mut self, op: BinaryOp, dst: &str, lhs: &str, rhs: &str) -> String;
    fn generate_return(&mut self, val_reg: Option<&str>) -> String;
    fn generate_load_immediate(&mut self, dst: &str, value: i32) -> String;
    fn generate_move(&mut self, dst: &str, src: &str) -> String;
}
