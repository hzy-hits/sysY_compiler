use anyhow::Context;
use koopa::ir::{BinaryOp, Value};

use crate::semantic::SymbolKind;
use crate::traits::semantic::{ConstEval, SymbolTable};

use crate::{ir_builder::IRBuilder, traits::to_ir::ExpToIr};

use super::refactor::{PrimaryExp, UnaryExp};
use super::ConstInitVal;
use super::{op::UnaryOp, refactor::Exp};
use super::{LVal, Result};

impl LVal {
    pub fn new(id: String) -> Self {
        Self { id }
    }
    pub fn get_address(&self, builder: &mut IRBuilder) -> Result<Value> {
        let sym = builder
            .lookup(&self.id)
            .with_context(|| format!("Undefined identifier: {}", self.id))?;
        match sym {
            SymbolKind::Variable(alloc) => Ok(*alloc),
            SymbolKind::Const(_) => Err(anyhow::anyhow!("Cannot assign to a constant {}", self.id)),
            _ => Err(anyhow::anyhow!("Not a variable")),
        }
    }

    pub fn load_value(&self, builder: &mut IRBuilder) -> Result<Value> {
        let sym = builder.lookup(&self.id)?;
        match sym {
            SymbolKind::Variable(alloc) => builder.create_load(*alloc),
            SymbolKind::Const(val) => Ok(builder.create_constant(*val)),
            _ => Err(anyhow::anyhow!("Not a variable")),
        }
    }
}

impl ExpToIr for Exp {
    fn to_ir(&self, builder: &mut IRBuilder) -> Result<Value> {
        match self {
            Exp::Primary(primary_exp) => primary_exp.to_ir(builder),
            Exp::UnaryExp(unary_exp) => unary_exp.to_ir(builder),
            Exp::Binary(lhs, op, rhs) => {
                let lhs_val = lhs.to_ir(builder)?;
                let rhs_val = rhs.to_ir(builder)?;

                match op {
                    BinaryOp::And | BinaryOp::Or => {
                        let logic_lhs = builder.to_logic_val(lhs_val)?;
                        let logic_rhs = builder.to_logic_val(rhs_val)?;
                        builder.create_binary(op, logic_lhs, logic_rhs)
                    }
                    _ => builder.create_binary(op, lhs_val, rhs_val),
                }
            }
        }
    }
}

impl ExpToIr for PrimaryExp {
    fn to_ir(&self, builder: &mut IRBuilder) -> Result<Value> {
        match self {
            PrimaryExp::Number(num) => Ok(builder.create_constant(*num)),
            PrimaryExp::Exp(exp) => exp.to_ir(builder),
            PrimaryExp::LVal(lval) => lval.load_value(builder),
        }
    }
}

impl ExpToIr for UnaryExp {
    fn to_ir(&self, builder: &mut IRBuilder) -> Result<Value> {
        match self {
            UnaryExp::PrimaryExp(primary_exp) => primary_exp.to_ir(builder),
            UnaryExp::UnaryOp(op, unary_exp) => {
                let val = unary_exp.to_ir(builder)?;
                match op {
                    UnaryOp::Plus => Ok(val),
                    UnaryOp::Minus => {
                        let zero = builder.create_constant(0);
                        builder.create_binary(&BinaryOp::Div, zero, val)
                    }
                    UnaryOp::Not => {
                        let zero = builder.create_constant(0);
                        builder.create_binary(&BinaryOp::Eq, val, zero)
                    }
                }
            }
        }
    }
}

impl ConstEval for LVal {
    fn eval_const(&self, builder: &IRBuilder) -> Result<i32> {
        let sym = builder.lookup(&self.id)?;
        match sym {
            SymbolKind::Const(num) => Ok(*num),
            _ => Err(anyhow::anyhow!("Not a constant")),
        }
    }
}

impl ConstEval for ConstInitVal {
    fn eval_const(&self, builder: &IRBuilder) -> Result<i32> {
        self.exp.eval_const(builder)
    }
}

impl ConstEval for Exp {
    fn eval_const(&self, builder: &IRBuilder) -> Result<i32> {
        match self {
            Exp::Primary(primary_exp) => primary_exp.eval_const(builder),
            Exp::UnaryExp(unary_exp) => unary_exp.eval_const(builder),
            Exp::Binary(lhs, op, rhs) => {
                let lhs_val = lhs.eval_const(builder)?;
                let rhs_val = rhs.eval_const(builder)?;

                match op {
                    BinaryOp::Add => Ok(lhs_val + rhs_val),
                    BinaryOp::Sub => Ok(lhs_val - rhs_val),
                    BinaryOp::Mul => Ok(lhs_val * rhs_val),
                    BinaryOp::Div => Ok(lhs_val / rhs_val),
                    BinaryOp::Mod => Ok(lhs_val % rhs_val),
                    BinaryOp::Eq => Ok((lhs_val == rhs_val) as i32),
                    BinaryOp::NotEq => Ok((lhs_val != rhs_val) as i32),
                    BinaryOp::Ge => Ok((lhs_val >= rhs_val) as i32),
                    BinaryOp::Gt => Ok((lhs_val > rhs_val) as i32),
                    BinaryOp::Le => Ok((lhs_val <= rhs_val) as i32),
                    BinaryOp::Lt => Ok((lhs_val < rhs_val) as i32),
                    BinaryOp::And => Ok(((lhs_val != 0) && (rhs_val != 0)) as i32),
                    BinaryOp::Or => Ok(((lhs_val != 0) || (rhs_val != 0)) as i32),
                    _ => Err(anyhow::anyhow!("Unsupported binary operation")),
                }
            }
        }
    }
}

impl ConstEval for PrimaryExp {
    fn eval_const(&self, builder: &IRBuilder) -> Result<i32> {
        match self {
            PrimaryExp::Number(num) => Ok(*num),
            PrimaryExp::Exp(exp) => exp.eval_const(builder),
            PrimaryExp::LVal(lval) => lval.eval_const(builder),
        }
    }
}

impl ConstEval for UnaryExp {
    fn eval_const(&self, builder: &IRBuilder) -> Result<i32> {
        match self {
            UnaryExp::PrimaryExp(primary_exp) => primary_exp.eval_const(builder),
            UnaryExp::UnaryOp(op, unary_exp) => {
                let val = unary_exp.eval_const(builder)?;
                match op {
                    UnaryOp::Plus => Ok(val),
                    UnaryOp::Minus => Ok(-val),
                    UnaryOp::Not => Ok((val == 0) as i32),
                }
            }
        }
    }
}
