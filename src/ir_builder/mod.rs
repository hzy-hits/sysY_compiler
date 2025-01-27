mod ir_builder_exp_op;
mod ir_builder_impl;

use koopa::ir::*;
use std::collections::HashMap;

use super::Result;
use crate::{asm_generator::AsmGenerator, ir_printer::IRPrinter, semantic::SymbolKind};
pub struct IRBuilder {
    program: Program,
    current_func: Option<Function>,
    current_block: Option<BasicBlock>,
    value_counter: usize,
    symbo_spaces: Vec<HashMap<String, SymbolKind>>,
    current_scope_level: usize,
}
// IRBuilder getter and new methods
impl IRBuilder {
    pub fn new() -> Self {
        Self {
            program: Program::new(),
            current_func: None,
            current_block: None,
            value_counter: 0,
            symbo_spaces: vec![HashMap::new()],
            current_scope_level: 0,
        }
    }
    fn next_value_id(&mut self) -> usize {
        let id = self.value_counter;
        self.value_counter += 1;
        id
    }
    pub fn program(&self) -> &Program {
        &self.program
    }

    pub fn program_mut(&mut self) -> &mut Program {
        &mut self.program
    }
    pub fn set_current_func(&mut self, func: Function) {
        self.current_func = Some(func);
    }

    pub fn set_current_bb(&mut self, bb: BasicBlock) {
        self.current_block = Some(bb);
    }

    pub fn create_int_type(&self) -> Type {
        Type::get_i32()
    }
    pub fn to_asm(&self, asm_gen: &mut AsmGenerator) -> String {
        asm_gen.generate_program(&self.program)
    }
    pub fn to_ir(&self, printer: &mut IRPrinter) -> String {
        printer.print_program(&self.program)
    }
    pub fn contains_var(&self, name: &str) -> bool {
        self.symbo_spaces.last().unwrap().contains_key(name)
    }
}
