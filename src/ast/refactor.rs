use koopa::ir::BinaryOp;

use super::UnaryOp;

#[derive(Debug)]
pub struct CompUnit {
    pub items: Vec<CompUnitItem>,
}

#[derive(Debug)]
pub enum CompUnitItem {
    Decl(Decl),
    FuncDef(FuncDef),
}

#[derive(Debug)]
pub struct FuncDef {
    pub func_type: FuncType,
    pub id: String,
    pub block: Block,
}

#[derive(Debug)]
pub struct Block {
    pub items: Vec<BlockItem>,
}

#[derive(Debug)]
pub enum BlockItem {
    Decl(Decl),
    Stmt(Stmt),
}

#[derive(Debug)]
pub enum Decl {
    ConstDecl(BType, Vec<ConstDef>),
    VarDecl(BType, Vec<VarDef>),
}

#[derive(Debug, Clone)]
pub enum BType {
    Int,
}

#[derive(Debug)]
pub struct ConstDef {
    pub id: String,
    pub value: ConstInitVal,
}

#[derive(Debug)]
pub struct ConstInitVal {
    pub exp: Box<Exp>,
}

#[derive(Debug)]
pub struct VarDef {
    pub id: String,
    pub ty: BType,
    pub init_val: Option<InitVal>,
}

#[derive(Debug)]
pub struct InitVal {
    pub exp: Box<Exp>,
}

#[derive(Debug)]
pub enum Stmt {
    Return(Option<Exp>),
    Exp(Option<Exp>),
    Block(Block),
    Assign(LVal, Exp),
}

#[derive(Debug, Clone)]
pub struct LVal {
    pub id: String,
}

#[derive(Debug, Clone)]
pub enum Exp {
    Primary(PrimaryExp),
    UnaryExp(Box<UnaryExp>),
    Binary(Box<Exp>, BinaryOp, Box<Exp>),
}

#[derive(Debug, Clone)]
pub enum UnaryExp {
    PrimaryExp(PrimaryExp),
    UnaryOp(UnaryOp, Box<UnaryExp>),
}

#[derive(Debug, Clone)]
pub enum PrimaryExp {
    Number(i32),
    LVal(LVal),
    Exp(Box<Exp>),
}

#[derive(Debug)]
pub enum FuncType {
    Int,
}
