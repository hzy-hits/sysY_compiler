use std::collections::HashMap;

use koopa::ir::Value;

#[derive(Default)]
pub struct RiscvRegisterManager {
    temp_regs: [bool; 7],   // t0-t6
    saved_regs: [bool; 12], // s0-s11
    arg_regs: [bool; 8],    // a0-a7
    pub(crate) value_reg_map: HashMap<Value, String>,
    pub(crate) value_use_count: HashMap<Value, usize>,
    stack_slots: HashMap<Value, i32>,
    current_stack_offset: i32,
}

impl RiscvRegisterManager {
    pub fn new() -> Self {
        Self {
            temp_regs: [false; 7],
            saved_regs: [false; 12],
            arg_regs: [false; 8],
            value_reg_map: HashMap::new(),
            value_use_count: HashMap::new(),
            stack_slots: HashMap::new(),
            current_stack_offset: 0,
        }
    }

    pub(crate) fn allocate_tmp(&mut self) -> Option<String> {
        for (i, used) in self.temp_regs.iter_mut().enumerate() {
            if !*used {
                *used = true;
                return Some(format!("t{}", i));
            }
        }
        None
    }
    pub(crate) fn allocate_saved(&mut self) -> Option<String> {
        for (i, used) in self.saved_regs.iter_mut().enumerate() {
            if !*used {
                *used = true;
                return Some(format!("s{}", i));
            }
        }
        None
    }

    pub(crate) fn free_register(&mut self, reg: &str) {
        let reg_type = reg.chars().next().unwrap();
        let reg_num: usize = reg[1..].parse().unwrap();

        match reg_type {
            't' => self.temp_regs[reg_num] = false,
            's' => self.saved_regs[reg_num] = false,
            'a' => self.arg_regs[reg_num] = false,
            _ => {}
        }
    }

    pub fn spill_to_stack(&mut self, val: Value) -> i32 {
        let offset = self.current_stack_offset;
        self.stack_slots.insert(val, offset);
        self.current_stack_offset += 4; // for risc v 32 bit
        offset
    }
    pub fn get_calling_convention_reg(&self, index: usize) -> Option<String> {
        if index < 8 {
            Some(format!("a{}", index))
        } else {
            None
        }
    }

    pub fn reserve_for_call(&mut self, arg_count: usize) {
        for i in 0..arg_count.min(8) {
            self.arg_regs[i] = true;
        }
    }
    pub fn get_return_reg(&self) -> String {
        "a0".to_string()
    }

    pub fn mark_register_live(&mut self, reg: &str) {
        let reg_type = reg.chars().next().unwrap();
        let reg_num: usize = reg[1..].parse().unwrap();

        match reg_type {
            't' => self.temp_regs[reg_num] = true,
            's' => self.saved_regs[reg_num] = true,
            'a' => self.arg_regs[reg_num] = true,
            _ => {}
        }
    }
    pub fn generate_prologue(&self) -> Vec<String> {
        let mut prologue = Vec::new();
        let mut to_save = Vec::new();

        for (i, &used) in self.saved_regs.iter().enumerate() {
            if used {
                to_save.push(format!("s{}", i));
            }
        }
        if to_save.len() > 0 {
            prologue.push(format!("  addi sp, sp, -{}", (to_save.len() * 4) as i32));
            for (i, reg) in to_save.iter().enumerate() {
                prologue.push(format!("  sw {}, {}(sp)", reg, i * 4));
            }
        }
        prologue
    }

    pub fn generate_epilogue(&self) -> Vec<String> {
        let mut epilogue = Vec::new();
        let mut to_restore = Vec::new();

        for (i, &used) in self.saved_regs.iter().enumerate() {
            if used {
                to_restore.push(format!("s{}", i));
            }
        }
        if to_restore.len() > 0 {
            for (i, reg) in to_restore.iter().enumerate() {
                epilogue.push(format!("  lw {}, {}(sp)", reg, i * 4));
            }
            epilogue.push(format!("  addi sp, sp, {}", (to_restore.len() * 4) as i32));
        }
        epilogue
    }
}
