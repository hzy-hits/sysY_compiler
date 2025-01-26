use std::collections::HashMap;

use koopa::ir::{
    builder::{BasicBlockBuilder, LocalInstBuilder, ValueBuilder},
    BasicBlock, Function, FunctionData, Type, Value,
};

use crate::{semantic::SymbolKind, traits::semantic::SymbolTable};

use super::IRBuilder;
use super::Result;
impl IRBuilder {
    pub fn create_function(
        &mut self,
        name: &str,
        params: Vec<(String, Type)>,
        return_type: Type,
    ) -> Function {
        let func = self.program.new_func(FunctionData::with_param_names(
            name.into(),
            params.into_iter().map(|(n, t)| (Some(n), t)).collect(),
            return_type,
        ));
        self.set_current_func(func);
        func
    }
    pub fn create_bb(&mut self, name: &str) -> Result<BasicBlock> {
        let func = self.current_func.expect("No active function");
        let bb = self
            .program
            .func_mut(func)
            .dfg_mut()
            .new_bb()
            .basic_block(Some(name.into()));
        self.program
            .func_mut(func)
            .layout_mut()
            .bbs_mut()
            .push_key_back(bb)
            .map_err(|_| anyhow::anyhow!("Failed to create basic block"))?;

        self.set_current_bb(bb);
        Ok(bb)
    }

    pub fn create_ret(&mut self, value: Option<Value>) {
        let func = self.current_func.expect("No active function");
        let bb = self.current_block.expect("No active basic block");
        let ret = self
            .program
            .func_mut(func)
            .dfg_mut()
            .new_value()
            .ret(Some(value).expect("No return value"));
        self.program
            .func_mut(func)
            .layout_mut()
            .bb_mut(bb)
            .insts_mut()
            .push_key_back(ret)
            .unwrap();
    }

    pub fn create_constant(&mut self, value: i32) -> Value {
        let func = self.current_func.expect("No active function");
        self.program
            .func_mut(func)
            .dfg_mut()
            .new_value()
            .integer(value)
    }

    pub fn value_type(&self, val: Value) -> Result<Type> {
        let func = self.current_func.expect("No active function");
        let ty = self.program.func(func).dfg().value(val).ty().clone();
        Ok(ty)
    }
}

impl SymbolTable for IRBuilder {
    fn enter_scope(&mut self) -> Result<()> {
        self.symbo_spaces.push(HashMap::new());
        self.current_scope_level += 1;
        Ok(())
    }

    fn exit_scope(&mut self) -> Result<()> {
        self.symbo_spaces
            .pop()
            .ok_or_else(|| anyhow::anyhow!("No scope to exit"))?;
        self.current_scope_level -= 1;

        Ok(())
    }

    fn lookup(&self, name: &str) -> Result<&SymbolKind> {
        for scope in self.symbo_spaces.iter().rev() {
            if let Some(sym) = scope.get(name) {
                return Ok(sym);
            }
        }
        Err(anyhow::anyhow!("Symbol {} not found", name))
    }

    fn add_symbol(&mut self, name: &str, kind: SymbolKind) -> Result<()> {
        let scope = self.symbo_spaces.last_mut().ok_or_else(|| {
            anyhow::anyhow!(
                "
No active scope"
            )
        })?;
        if scope.contains_key(name) {
            return Err(anyhow::anyhow!("Symbol {} already exists", name));
        }
        scope.insert(name.to_string(), kind);
        Ok(())
    }

    fn current_scope_level(&self) -> usize {
        self.current_scope_level
    }
}
