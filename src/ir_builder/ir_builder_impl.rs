use koopa::ir::{
    builder::{BasicBlockBuilder, LocalInstBuilder, ValueBuilder},
    BasicBlock, Function, FunctionData, Type, Value,
};

use super::IRBuilder;

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
    pub fn create_bb(&mut self, name: &str) -> Result<BasicBlock, String> {
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
            .map_err(|_| "Failed to create basic block".to_string())?;
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
}
