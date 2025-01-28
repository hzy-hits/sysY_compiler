use anyhow::ensure;
use koopa::ir::{builder::LocalInstBuilder, BinaryOp, Value};
use koopa::ir::{BasicBlock, Function, Type, ValueKind};

use super::IRBuilder;
use super::Result;
impl IRBuilder {
    fn get_current_context(&self) -> Result<(Function, BasicBlock)> {
        let func = self
            .current_func
            .ok_or_else(|| anyhow::anyhow!("No active function"))?;
        let bb = self
            .current_block
            .ok_or_else(|| anyhow::anyhow!("No active basic block"))?;
        Ok((func, bb))
    }
    fn create_instruction<F>(&mut self, create_value: F) -> Result<Value>
    where
        F: FnOnce(&mut koopa::ir::dfg::DataFlowGraph) -> Value,
    {
        let (func, bb) = self.get_current_context()?;
        let dfg = self.program.func_mut(func).dfg_mut();

        let value = create_value(dfg);

        if let Some(val_data) = dfg.values().get(&value) {
            if val_data.name().is_none() && needs_id(val_data.kind()) {
                let id = self.next_value_id();
                self.program
                    .func_mut(func)
                    .dfg_mut()
                    .set_value_name(value, Some(format!("%{}", id)));
            }
        }
        self.program
            .func_mut(func)
            .layout_mut()
            .bb_mut(bb)
            .insts_mut()
            .push_key_back(value)
            .map_err(|_| anyhow::anyhow!("Failed to insert instruction"))?;

        Ok(value)
    }

    pub fn create_binary(&mut self, op: &BinaryOp, lhs: Value, rhs: Value) -> Result<Value> {
        let (func, _) = self.get_current_context()?;

        let lhs_ty = self.program.func(func).dfg().value(lhs).ty();
        let rhs_ty = self.program.func(func).dfg().value(rhs).ty();
        ensure!(
            lhs_ty == rhs_ty,
            "Type mismatch in binary op: {:?} vs {:?}",
            lhs_ty,
            rhs_ty
        );

        let real_op = *op;
        Ok(self.create_instruction(|dfg| dfg.new_value().binary(real_op, lhs, rhs))?)
    }

    pub(crate) fn to_logic_val(&mut self, val: Value) -> Result<Value> {
        let zero = self.create_constant(0);
        let inner = self.create_binary(&BinaryOp::Eq, zero, val)?;
        self.create_binary(&BinaryOp::Eq, zero, inner)
    }

    pub fn create_alloc(&mut self, ty: Type, name: String) -> Result<Value> {
        self.create_instruction(|dfg| {
            let alloc = dfg.new_value().alloc(ty);
            dfg.set_value_name(alloc, Some(name));
            alloc
        })
    }

    pub fn create_store(&mut self, ptr: Value, value: Value) -> Result<()> {
        self.create_instruction(|dfg| dfg.new_value().store(value, ptr))?;
        Ok(())
    }
    pub fn create_load(&mut self, ptr: Value) -> Result<Value> {
        self.create_instruction(|dfg| dfg.new_value().load(ptr))
    }
}

fn needs_id(kind: &ValueKind) -> bool {
    matches!(kind, ValueKind::Load(_) | ValueKind::Binary(_))
}
