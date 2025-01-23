use std::fmt::Write;

use koopa::ir::{
    values::{Binary, Return},
    BinaryOp, FunctionData, Value, ValueKind,
};

use crate::traits::instruct_generator::InstructionGenerator;

use super::AsmGenerator;

impl AsmGenerator {
    // fn update_reg_usage(&mut self, val: Value, reg: &str) {
    //     self.reg_manager.after_value_use(val);
    // }

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
                if let Some(reg) = self.reg_manager.allocate_tmp() {
                    self.output.push_str(
                        self.inst_generator
                            .generate_load_immediate(&reg, c.value())
                            .as_str(),
                    );
                    self.reg_manager.value_reg_map.insert(val, reg.clone());
                    return reg;
                }
                unreachable!("No available registers");
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
        if lhs_reg != "x0" {
            self.reg_manager.after_value_use(binary.lhs());
        }
        if rhs_reg != "x0" {
            self.reg_manager.after_value_use(binary.rhs());
        }
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

        let dst_reg = if self.can_reuse_register(lhs) {
            self.reg_manager.value_reg_map.remove(&lhs);
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

    pub fn init_function(&mut self, func: &FunctionData) {
        self.reg_manager.value_reg_map.clear();
        self.reg_manager.value_use_count.clear();
        self.reg_manager.stack_slots.clear();
        self.reg_manager.current_stack_offset = 0;

        let prologue = self.reg_manager.generate_prologue();
        self.output
            .extend(prologue.iter().map(|s| format!("{}\n", s)));
    }
}
