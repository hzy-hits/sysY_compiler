use super::Result;
use crate::{ir_builder::IRBuilder, semantic::SymbolKind};
pub trait ConstEval {
    fn eval_const(&self, builder: &IRBuilder) -> Result<i32>;
}

pub trait SymbolTable {
    fn enter_scope(&mut self) -> Result<()>;
    fn exit_scope(&mut self) -> Result<()>;
    fn lookup(&self, name: &str) -> Result<&SymbolKind>;
    fn add_symbol(&mut self, name: &str, kind: SymbolKind) -> Result<()>;
    fn current_scope_level(&self) -> usize;
}
