use koopa::ir::{Function, Value};
#[derive(Debug, Clone)]
pub enum SymbolKind {
    Const { value: i32, scope_level: usize },
    Variable { value: Value, scope_level: usize },
    Function { func: Function, scope_level: usize },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
}

impl SymbolKind {
    pub fn get_scope_level(&self) -> usize {
        match self {
            SymbolKind::Const { scope_level, .. } => *scope_level,
            SymbolKind::Variable { scope_level, .. } => *scope_level,
            SymbolKind::Function { scope_level, .. } => *scope_level,
        }
    }
}
