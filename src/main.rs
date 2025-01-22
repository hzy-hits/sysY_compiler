use lalrpop_util::lalrpop_mod;
use std::env::args;
use std::fs::read_to_string;
use std::fs::write;
use std::io::Result;
use sysY::asm_generator;
use sysY::traits::ToIr;
use sysY::{ir_builder, ir_printer};
// 引用 lalrpop 生成的解析器
// 因为我们刚刚创建了 sysy.lalrpop, 所以模块名是 sysy

lalrpop_mod!(sysy);
fn main() -> Result<()> {
    // 解析命令行参数
    let mut args = args();
    args.next();
    let mode = args.next().unwrap();
    println!("{}", mode);
    let input = args.next().unwrap();
    args.next();
    let output = args.next().unwrap();

    // 读取输入文件
    let input = read_to_string(input)?;

    // 调用 lalrpop 生成的 parser 解析输入文件
    let ast = sysy::CompUnitParser::new().parse(&input).unwrap();
    let mut builder = ir_builder::IRBuilder::new();

    ast.to_ir(&mut builder).unwrap();
    match mode.as_str() {
        "-ir" => {
            let mut printer = ir_printer::IRPrinter::new();
            let ir = builder.to_ir(&mut printer);
            write(&output, ir)?;
        }

        "-riscv" => {
            let mut code_generator = asm_generator::AsmGenerator::new();
            let res = builder.to_asm(&mut code_generator);
            write(&output, res)?;
        }
        _ => {
            unreachable!("Invalid mode");
        }
    }

    println!("{:#?}", ast);

    Ok(())
}
