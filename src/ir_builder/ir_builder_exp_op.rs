use koopa::ir::{builder::LocalInstBuilder, BinaryOp, Value};

use super::IRBuilder;

impl IRBuilder {
    pub fn create_binary(&mut self, op: &str, lhs: Value, rhs: Value) -> Value {
        let func = self.current_func.expect("No active function");
        let bb = self.current_block.expect("No active basic block");
        let id = self.next_value_id();
        let real_op = match op {
            "add" => BinaryOp::Add,
            "sub" => BinaryOp::Sub,
            "mul" => BinaryOp::Mul,
            "div" => BinaryOp::Div,
            "mod" => BinaryOp::Mod,
            "eq" => BinaryOp::Eq,
            "lt" => BinaryOp::Lt,
            "gt" => BinaryOp::Gt,
            "le" => BinaryOp::Le,
            "ge" => BinaryOp::Ge,
            "ne" => BinaryOp::NotEq,
            "or" => BinaryOp::Or,
            "and" => BinaryOp::And,

            _ => panic!("Unknown binary operator: {}", op),
        };
        let value = self
            .program
            .func_mut(func)
            .dfg_mut()
            .new_value()
            .binary(real_op, lhs, rhs);
        self.program
            .func_mut(func)
            .dfg_mut()
            .set_value_name(value, Some(format!("%{}", id)));
        self.program
            .func_mut(func)
            .layout_mut()
            .bb_mut(bb)
            .insts_mut()
            .push_key_back(value)
            .unwrap();

        value
    }

    pub(crate) fn to_logic_val(&mut self, val: Value) -> Value {
        let zero = self.create_constant(0);
        let inner = self.create_binary("eq", zero, val);
        self.create_binary("eq", zero, inner)
    }
}
