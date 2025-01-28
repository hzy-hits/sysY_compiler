use koopa::ir::Value;

use crate::semantic::SymbolKind;
use crate::traits::semantic::SymbolTable;

use super::IRBuilder;
use super::Result;
impl IRBuilder {
    fn get_scoped_name(&self, name: &str) -> String {
        format!("@{}_{}", name, self.current_scope_level)
    }

    pub fn create_variable(&mut self, name: &str, init_val: Option<Value>) -> Result<Value> {
        let scoped_name = self.get_scoped_name(name);
        let value = self.create_alloc(self.create_int_type(), scoped_name)?;

        let symbol = SymbolKind::Variable {
            value,
            scope_level: self.current_scope_level,
        };
        self.add_symbol(name, symbol)?;

        if let Some(init_val) = init_val {
            self.create_store(value, init_val)?;
        }
        Ok(value)
    }

    pub fn create_const(&mut self, name: &str, value: i32) -> Result<()> {
        let scoped_name = self.get_scoped_name(name);
        let symbol = SymbolKind::Const {
            value: value,
            scope_level: self.current_scope_level,
        };

        self.add_symbol(&scoped_name, symbol)
    }

    pub fn get_var_value(&self, name: &str) -> Result<Value> {
        let symbol = self.lookup(name)?;
        match symbol {
            SymbolKind::Variable { value, .. } => Ok(*value),
            _ => Err(anyhow::anyhow!("Not a variable: {}", name)),
        }
    }

    pub fn load_var(&mut self, name: &str) -> Result<Value> {
        let var_value = self.get_var_value(name)?;
        self.create_load(var_value)
    }

    pub fn store_var(&mut self, name: &str, value: Value) -> Result<()> {
        let var_value = self.get_var_value(name)?;
        self.create_store(var_value, value)
    }

    pub fn contains_var_in_current_scope(&self, name: &str) -> bool {
        match self.lookup(name) {
            Ok(SymbolKind::Variable { scope_level, .. }) => {
                *scope_level == self.current_scope_level
            }
            _ => false,
        }
    }
}
