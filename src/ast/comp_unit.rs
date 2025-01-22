use crate::{ast::FuncDef, ir_builder::IRBuilder, traits::ToIr};

#[derive(Debug)]
pub struct CompUnit {
    pub func_def: FuncDef,
}

impl ToIr for CompUnit {
    fn to_ir(&self, builder: &mut IRBuilder) -> Result<(), String> {
        self.func_def.to_ir(builder)
    }
}
