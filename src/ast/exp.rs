use koopa::ir::Value;

use crate::{ir_builder::IRBuilder, traits::to_ir::ExpToIr};

use super::op::UnaryOp;

#[derive(Debug, Clone)]
pub enum Exp {
    UnaryExp(Box<UnaryExp>),
}

impl ExpToIr for Exp {
    fn to_ir(&self, builder: &mut IRBuilder) -> Result<Value, String> {
        match self {
            Exp::UnaryExp(unary_exp) => unary_exp.to_ir(builder),
        }
    }
}

#[derive(Debug, Clone)]
pub enum PrimaryExp {
    Number(i32),
    Exp(Box<Exp>),
}

impl ExpToIr for PrimaryExp {
    fn to_ir(&self, builder: &mut IRBuilder) -> Result<Value, String> {
        match self {
            PrimaryExp::Number(num) => Ok(builder.create_constant(*num)),
            PrimaryExp::Exp(exp) => exp.to_ir(builder),
        }
    }
}

#[derive(Debug, Clone)]
pub enum UnaryExp {
    PrimaryExp(PrimaryExp),
    UnaryOp(UnaryOp, Box<UnaryExp>),
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
                        Ok(builder.create_binary("sub", zero, val))
                    }
                    UnaryOp::Not => {
                        let zero = builder.create_constant(0);
                        Ok(builder.create_binary("eq", val, zero))
                    }
                }
            }
        }
    }
}
