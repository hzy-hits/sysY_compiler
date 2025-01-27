use std::fmt::Write;

use koopa::ir::{
    values::{Binary, Return, Store},
    BinaryOp, FunctionData, Value, ValueKind,
};

use crate::traits::instruct_generator::InstructionGenerator;

use super::AsmGenerator;

impl AsmGenerator {
    // fn update_reg_usage(&mut self, val: Value, reg: &str) {
    //     self.reg_manager.after_value_use(val);
    // }

    fn get_or_generate_value_reg(&mut self, func: &FunctionData, val: Value) -> String {
        if let Some(&offset) = self.reg_manager.stack_slots.get(&val) {
            let reg = match self.reg_manager.allocate_tmp() {
                Some(r) => r,
                None => self
                    .spill_and_get_reg(func, val)
                    .expect("Failed to allocate register even after spilling"),
            };
            self.output
                .push_str(&format!("  lw {}, {}(sp)\n", reg, offset));
            self.reg_manager.value_reg_map.insert(val, reg.clone());
            return reg;
        }
        if let Some(reg) = self.reg_manager.value_reg_map.get(&val) {
            reg.clone()
        } else {
            self.generate_value_and_get_reg(func, val)
        }
    }

    fn find_reg_to_spill(&self) -> Option<(Value, String)> {
        let mut candidate = None;
        let mut max_next_use = 0;

        for (old_val, reg) in self.reg_manager.value_reg_map.iter() {
            if reg.starts_with('t') {
                if let Some(use_count) = self.reg_manager.value_use_count.get(old_val) {
                    if *use_count > 0 && *use_count > max_next_use {
                        max_next_use = *use_count;
                        candidate = Some((*old_val, reg.clone()));
                    }
                }
            }
        }
        candidate
    }

    fn spill_and_get_reg(&mut self, func: &FunctionData, val: Value) -> Option<String> {
        let (old_val, reg) = self.find_reg_to_spill()?;
        let offset = self.reg_manager.spill_to_stack(old_val);
        self.output
            .push_str(&format!("  sw {}, {}(sp)\n", &reg, offset));
        self.reg_manager.free_register(&reg, old_val);
        self.reg_manager.value_reg_map.remove(&old_val);
        Some(reg)
    }
    fn generate_value_and_get_reg(&mut self, func: &FunctionData, val: Value) -> String {
        let data = func.dfg().value(val);
        match data.kind() {
            ValueKind::Integer(c) => {
                if c.value() == 0 {
                    return "x0".to_string();
                }
                let reg = match self.reg_manager.allocate_tmp() {
                    Some(r) => r,
                    None => self
                        .spill_and_get_reg(func, val)
                        .expect("Failed to allocate register even after spilling"),
                };

                self.output.push_str(
                    self.inst_generator
                        .generate_load_immediate(&reg, c.value())
                        .as_str(),
                );
                self.reg_manager.value_reg_map.insert(val, reg.clone());
                reg
            }

            ValueKind::Alloc(_) => {
                println!("Generating code for Alloc value: {:?}", val);
                println!("Current stack slots: {:?}", self.reg_manager.stack_slots);
                let offset =
                    self.reg_manager.stack_slots.get(&val).copied().expect(
                        "Alloc value must have a stack slot assigned during initialization",
                    );
                let reg = match self.reg_manager.allocate_tmp() {
                    Some(r) => r,
                    None => self
                        .spill_and_get_reg(func, val)
                        .expect("Failed to allocate register even after spilling"),
                };

                self.output
                    .push_str(&format!("  addi {}, {}, {}\n", reg, "sp", offset));
                self.reg_manager.value_reg_map.insert(val, reg.clone());
                reg
            }

            ValueKind::Load(load) => {
                let src = load.src();
                if let Some(&offset) = self.reg_manager.stack_slots.get(&src) {
                    let dst_reg = self
                        .reg_manager
                        .allocate_tmp()
                        .or_else(|| self.spill_and_get_reg(func, val))
                        .expect("Failed to allocate register");
                    self.output
                        .push_str(&format!("  lw {}, {}(sp)\n", dst_reg, offset));
                    self.reg_manager.value_reg_map.insert(val, dst_reg.clone());
                    return dst_reg;
                }
                unreachable!("Load source must be allocated");
            }

            _ => {
                unreachable!("Unsupported value kind {:?}", data.kind());
            }
        }
    }

    fn handle_return(&mut self, func: &FunctionData, ret: &Return) {
        if let Some(val) = ret.value() {
            if let Some(offset) = self.reg_manager.stack_slots.get(&val) {
                self.output.push_str(&format!("  lw a0, {}(sp)\n", offset));
            } else {
                let reg = self.get_or_generate_value_reg(func, val);
                if reg != "a0" {
                    self.output.push_str(&format!("  mv a0, {}\n", reg));
                }
            }
        }

        let epilogue = self.reg_manager.generate_epilogue();
        self.output
            .extend(epilogue.iter().map(|s| format!("{}\n", s)));
        self.output.push_str("  ret\n");
    }

    fn handle_binary(&mut self, func: &FunctionData, val: Value, binary: &Binary) {
        let (dst_reg, lhs_reg, rhs_reg) = self.prepare_binary_ops(func, binary);

        let inst = self
            .inst_generator
            .generate_binary(binary.op(), &dst_reg, &lhs_reg, &rhs_reg);
        self.output.push_str(&inst);

        if let Some(offset) = self.reg_manager.stack_slots.get(&val) {
            self.output
                .push_str(&format!("  sw {}, {}({})\n", dst_reg, offset, "sp"));
            self.reg_manager.free_register(&dst_reg, val);
        }

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

        let dst_reg = if self.can_reuse_register(lhs) && lhs_reg != "x0" {
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
            ValueKind::Store(store) => self.handle_store(func, store),
            _ => {}
        }
    }

    fn handle_store(&mut self, func: &FunctionData, store: &Store) {
        let val_reg = self.get_or_generate_value_reg(func, store.value());
        let dest = store.dest();
        if let Some(&offset) = self.reg_manager.stack_slots.get(&dest) {
            self.output
                .push_str(&format!("  sw {}, {}(sp)\n", val_reg, offset));
        }
        self.reg_manager.after_value_use(store.value());
        self.reg_manager.after_value_use(store.dest());
    }

    pub fn init_function(&mut self, func: &FunctionData) {
        self.reg_manager.reset_stack();
        let mut stack_size = 0;
        for (bb, _) in func.layout().bbs() {
            let bb_node = func.layout().bbs().node(&bb).unwrap();
            for (inst, _) in bb_node.insts() {
                let data = func.dfg().value(*inst);
                match data.kind() {
                    ValueKind::Alloc(_) => {
                        println!("Allocating stack for Alloc: {:?}", inst);
                        self.reg_manager.stack_slots.insert(*inst, stack_size);
                        stack_size += 4;
                    }

                    _ => {}
                }
            }
        }

        let aligned_size = (stack_size + 15) / 16 * 16;
        self.reg_manager.current_stack_offset = aligned_size;
        println!(
            "Stack slots after initialization: {:?}",
            self.reg_manager.stack_slots
        );

        self.reg_manager.reset_registers();

        let prologue = self.reg_manager.generate_prologue();
        self.output
            .extend(prologue.iter().map(|s| format!("{}\n", s)));
    }
}
