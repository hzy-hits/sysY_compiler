use std::fmt::Write;

use koopa::ir::{FunctionData, Program, Value, ValueKind};

use super::AsmGenerator;

impl AsmGenerator {
    pub fn generate_program(&mut self, program: &Program) -> String {
        self.output.clear();

        writeln!(&mut self.output, ".text").unwrap();

        for func in program.func_layout() {
            self.generate_function(program.func(*func));
        }
        self.output.clone()
    }

    pub fn generate_function(&mut self, func: &FunctionData) {
        self.count_value_uses(func);
        let func_name = func.name().strip_prefix("@").unwrap();
        writeln!(&mut self.output, ".global {}", func_name).unwrap(); // .global function
        writeln!(&mut self.output, "{}:", func_name).unwrap(); // function
        for (bb, _) in func.layout().bbs() {
            for (inst, _) in func.layout().bbs().node(bb).unwrap().insts() {
                self.generate_instruction(func, *inst);
            }
        }
    }

    fn count_value_uses(&mut self, func: &FunctionData) {
        self.reg_manager.value_use_count.clear();

        for (bb, _) in func.layout().bbs() {
            let bb_node = func.layout().bbs().node(bb).unwrap();
            for (inst, _) in bb_node.insts() {
                let data = func.dfg().value(*inst);

                match data.kind() {
                    ValueKind::Binary(binary) => {
                        self.increment_use_count(binary.lhs());
                        self.increment_use_count(binary.rhs());
                    }

                    ValueKind::Return(ret) => {
                        if let Some(val) = ret.value() {
                            self.increment_use_count(val);
                        }
                    }

                    _ => {}
                }
            }
        }
    }

    fn increment_use_count(&mut self, val: Value) {
        *self.reg_manager.value_use_count.entry(val).or_insert(0) += 1;
    }

    fn get_use_count(&self, val: &Value) -> usize {
        self.reg_manager
            .value_use_count
            .get(val)
            .copied()
            .unwrap_or(0)
    }

    fn is_last_use(&self, val: &Value) -> bool {
        self.get_use_count(val) <= 1
    }

    fn decrease_use_count(&mut self, val: &Value) {
        if let Some(count) = self.reg_manager.value_use_count.get_mut(val) {
            if *count > 0 {
                *count -= 1;
            }
        }
    }
}
