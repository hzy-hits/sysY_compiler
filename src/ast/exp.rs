use koopa::ir::Value;

use crate::{ir_builder::IRBuilder, traits::to_ir::ExpToIr};

use super::op::{Op, UnaryOp};

#[derive(Debug, Clone)]
pub enum Exp {
    UnaryExp(Box<UnaryExp>),
    Binary(Box<Exp>, Op, Box<Exp>),
}

impl ExpToIr for Exp {
    fn to_ir(&self, builder: &mut IRBuilder) -> Result<Value, String> {
        match self {
            Exp::UnaryExp(unary_exp) => unary_exp.to_ir(builder),
            Exp::Binary(lhs, op, rhs) => {
                let lhs_val = lhs.to_ir(builder)?;
                let rhs_val = rhs.to_ir(builder)?;
                match op {
                    Op::Add => Ok(builder.create_binary("add", lhs_val, rhs_val)),
                    Op::Sub => Ok(builder.create_binary("sub", lhs_val, rhs_val)),
                    Op::Mul => Ok(builder.create_binary("mul", lhs_val, rhs_val)),
                    Op::Div => Ok(builder.create_binary("div", lhs_val, rhs_val)),
                    Op::Mod => Ok(builder.create_binary("mod", lhs_val, rhs_val)),
                    Op::Eq => Ok(builder.create_binary("eq", lhs_val, rhs_val)),
                    Op::Lt => Ok(builder.create_binary("lt", lhs_val, rhs_val)),
                    Op::Gt => Ok(builder.create_binary("gt", lhs_val, rhs_val)),
                    Op::Le => Ok(builder.create_binary("le", lhs_val, rhs_val)),
                    Op::Ge => Ok(builder.create_binary("ge", lhs_val, rhs_val)),
                    Op::Ne => Ok(builder.create_binary("ne", lhs_val, rhs_val)),
                    Op::And => {
                        let logic_lhs_val = builder.to_logic_val(lhs_val);
                        let logic_rhs_val = builder.to_logic_val(rhs_val);
                        Ok(builder.create_binary("and", logic_lhs_val, logic_rhs_val))
                    }
                    Op::Or => {
                        let logic_lhs_val = builder.to_logic_val(lhs_val);
                        let logic_rhs_val = builder.to_logic_val(rhs_val);
                        Ok(builder.create_binary("or", logic_lhs_val, logic_rhs_val))
                    }
                }
            }
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

pub enum MulExp {
    UnaryExp(Box<UnaryExp>),
    Mul(Box<MulExp>, Box<UnaryExp>),
    Div(Box<MulExp>, Box<UnaryExp>),
}

pub enum AddExp {
    MulExp(Box<MulExp>),
}
