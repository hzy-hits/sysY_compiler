use koopa::ir::{Function, Value};
#[derive(Debug, Clone)]
pub enum SymbolKind {
    Const(i32),
    Variable(Value),
    Function(Function),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
}
