#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Plus,  // "+"
    Minus, // "-"
    Not,   // "!"
}

#[derive(Debug, Clone, PartialEq)]
pub enum Op {
    // MulExp level
    Mul, // "*"
    Div, // "/"
    Mod, // "%"

    // AddExp level
    Add, // "+"
    Sub, // "-"

    // RelExp level
    Lt, // "<"
    Gt, // ">"
    Le, // "<="
    Ge, // ">="

    // EqExp level
    Eq, // "=="
    Ne, // "!="

    // LAndExp level
    And, // "&&"

    // LOrExp level
    Or, // "||"
}

impl Op {
    // 获取操作符优先级
    pub fn precedence(&self) -> u8 {
        match self {
            // 数字越大优先级越高
            Op::Mul | Op::Div | Op::Mod => 6,
            Op::Add | Op::Sub => 5,
            Op::Lt | Op::Gt | Op::Le | Op::Ge => 4,
            Op::Eq | Op::Ne => 3,
            Op::And => 2,
            Op::Or => 1,
        }
    }

    pub fn to_ir_op(&self) -> &'static str {
        match self {
            Op::Add => "add",
            Op::Sub => "sub",
            Op::Mul => "mul",
            Op::Div => "div",
            Op::Mod => "mod",
            Op::Lt => "lt",
            Op::Gt => "gt",
            Op::Le => "le",
            Op::Ge => "ge",
            Op::Eq => "eq",
            Op::Ne => "ne",
            Op::And => "and",
            Op::Or => "or",
        }
    }
}
