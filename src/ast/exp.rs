use koopa::ir::{BinaryOp, Value};

use crate::{ir_builder::IRBuilder, traits::to_ir::ExpToIr};

use super::op::Op;
use super::refactor::{PrimaryExp, UnaryExp};
use super::{op::UnaryOp, refactor::Exp};

impl ExpToIr for Exp {
    fn to_ir(&self, builder: &mut IRBuilder) -> Result<Value, String> {
        match self {
            Exp::Primary(primary_exp) => primary_exp.to_ir(builder),
            Exp::UnaryExp(unary_exp) => unary_exp.to_ir(builder),
            Exp::Binary(lhs, op, rhs) => {
                let lhs_val = lhs.to_ir(builder)?;
                let rhs_val = rhs.to_ir(builder)?;

                match op {
                    BinaryOp::And | BinaryOp::Or => {
                        let logic_lhs = builder.to_logic_val(lhs_val);
                        let logic_rhs = builder.to_logic_val(rhs_val);
                        Ok(builder.create_binary(op, logic_lhs, logic_rhs))
                    }
                    _ => Ok(builder.create_binary(op, lhs_val, rhs_val)),
                }
            }
        }
    }
}

impl ExpToIr for PrimaryExp {
    fn to_ir(&self, builder: &mut IRBuilder) -> Result<Value, String> {
        match self {
            PrimaryExp::Number(num) => Ok(builder.create_constant(*num)),
            PrimaryExp::Exp(exp) => exp.to_ir(builder),
            PrimaryExp::LVal(_) => unimplemented!(),
        }
    }
}

impl ExpToIr for UnaryExp {
    fn to_ir(&self, builder: &mut IRBuilder) -> Result<Value, String> {
        match self {
            UnaryExp::PrimaryExp(primary_exp) => primary_exp.to_ir(builder),
            UnaryExp::UnaryOp(op, unary_exp) => {
                let val = unary_exp.to_ir(builder)?;
                match op {
                    UnaryOp::Plus => Ok(val),
                    UnaryOp::Minus => {
                        let zero = builder.create_constant(0);
                        Ok(builder.create_binary(&BinaryOp::Div, zero, val))
                    }
                    UnaryOp::Not => {
                        let zero = builder.create_constant(0);
                        Ok(builder.create_binary(&BinaryOp::Eq, val, zero))
                    }
                }
            }
        }
    }
}
