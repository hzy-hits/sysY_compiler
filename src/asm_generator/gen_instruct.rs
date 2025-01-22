use std::fmt::Write;

use koopa::ir::{
    values::{Binary, Return},
    BinaryOp, FunctionData, Value, ValueKind,
};

use crate::traits::instruct_generator::InstructionGenerator;

use super::AsmGenerator;

impl AsmGenerator {
    fn update_reg_usage(&mut self, val: Value, reg: &str) {
        if let Some(count) = self.reg_manager.value_use_count.get_mut(&val) {
            *count = count.saturating_sub(1);
            if *count == 0 {
                self.reg_manager.free_register(reg);
            }
        }
    }
    fn get_or_generate_value_reg(&mut self, func: &FunctionData, val: Value) -> String {
        if let Some(reg) = self.reg_manager.value_reg_map.get(&val) {
            reg.clone()
        } else {
            self.generate_value_and_get_reg(func, val)
        }
    }
    fn generate_value_and_get_reg(&mut self, func: &FunctionData, val: Value) -> String {
        let data = func.dfg().value(val);
        match data.kind() {
            ValueKind::Integer(c) => {
                if c.value() == 0 {
                    return "x0".to_string();
                }

                // allocate a temporary register from the register manager
                let reg = self
                    .reg_manager
                    .allocate_tmp()
                    .expect("No more temporary registers");
                self.output.push_str(
                    self.inst_generator
                        .generate_load_immediate(&reg, c.value())
                        .as_str(),
                );
                reg
            }
            _ => {
                unreachable!("Unsupported value kind");
            }
        }
    }

    fn handle_return(&mut self, func: &FunctionData, ret: &Return) {
        if let Some(val) = ret.value() {
            let reg = self.get_or_generate_value_reg(func, val);
            let ret_string = self.inst_generator.generate_return(Some(&reg));
            self.output.push_str(&ret_string);
        } else {
            self.output
                .push_str(&self.inst_generator.generate_return(None));
        }
    }

    fn handle_binary(&mut self, func: &FunctionData, val: Value, binary: &Binary) {
        let (dst_reg, lhs_reg, rhs_reg) = self.prepare_binary_ops(func, binary);

        let inst = self
            .inst_generator
            .generate_binary(binary.op(), &dst_reg, &lhs_reg, &rhs_reg);
        self.output.push_str(&inst);
        self.reg_manager.value_reg_map.insert(val, dst_reg);
        self.update_reg_usage(binary.lhs(), &lhs_reg);
        self.update_reg_usage(binary.rhs(), &rhs_reg);
    }

    fn can_reuse_register(&self, val: Value) -> bool {
        self.reg_manager
            .value_use_count
            .get(&val)
            .map_or(false, |&count| count <= 1)
    }

    fn prepare_binary_ops(
        &mut self,
        func: &FunctionData,
        binary: &Binary,
    ) -> (String, String, String) {
        let lhs = binary.lhs();
        let rhs = binary.rhs();
        let lhs_reg = self.get_or_generate_value_reg(func, lhs);
        let rhs_reg = self.get_or_generate_value_reg(func, rhs);
        let can_reuse_lhs = self.can_reuse_register(lhs);
        let dst_reg = if can_reuse_lhs && lhs_reg != "x0" {
            lhs_reg.clone()
        } else {
            self.reg_manager.allocate_tmp().unwrap()
        };
        (dst_reg, lhs_reg, rhs_reg)
    }

    pub fn generate_instruction(&mut self, func: &FunctionData, val: Value) {
        let data = func.dfg().value(val);
        match data.kind() {
            ValueKind::Return(ret) => self.handle_return(func, ret),
            ValueKind::Binary(binary) => self.handle_binary(func, val, binary),
            _ => {}
        }
    }
}
