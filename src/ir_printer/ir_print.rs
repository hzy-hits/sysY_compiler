use super::IRPrinter;
use koopa::ir::{BasicBlock, FunctionData, Program, Value, ValueKind};
use std::fmt::Write;

impl IRPrinter {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            indent_level: 0,
        }
    }

    pub fn print_program(&mut self, program: &Program) -> String {
        self.output.clear();

        // Print each function
        for &func in program.func_layout() {
            self.print_function(program.func(func));
            self.output.push('\n');
        }

        self.output.clone()
    }

    fn print_function(&mut self, func: &FunctionData) {
        // Print function header
        writeln!(&mut self.output, "fun {} {} {{", func.name(), func.ty()).unwrap();

        self.indent_level += 1;

        // Print each basic block
        for (bb, _) in func.layout().bbs() {
            self.print_basic_block(func, *bb);
        }

        self.indent_level -= 1;
        self.output.push_str("}\n");
    }

    fn print_basic_block(&mut self, func: &FunctionData, bb: BasicBlock) {
        // Print block label
        let bb_data = func.dfg().bb(bb);
        let name = bb_data
            .name()
            .as_ref()
            .map_or("".to_string(), |n| n.clone());
        self.indent();
        writeln!(&mut self.output, "{}:", name).unwrap();

        self.indent_level += 1;

        // Print instructions
        let bb_node = func
            .layout()
            .bbs()
            .node(&bb)
            .expect("Basic block not found");
        for (inst, _) in bb_node.insts() {
            self.print_instruction(func, *inst);
        }
        self.indent_level -= 1;
    }

    fn print_instruction(&mut self, func: &FunctionData, value: Value) {
        let data = func.dfg().value(value);

        self.indent();
        match data.kind() {
            ValueKind::Return(ret) => {
                if let Some(val) = ret.value() {
                    let val = self.value_to_string(func, val);
                    writeln!(&mut self.output, "ret {}", val).unwrap();
                } else {
                    writeln!(&mut self.output, "ret").unwrap();
                }
            }
            ValueKind::Integer(int) => {
                writeln!(&mut self.output, "{}", int.value()).unwrap();
            }
            ValueKind::Binary(bin) => {
                let lhs = self.value_to_string(func, bin.lhs());
                let rhs = self.value_to_string(func, bin.rhs());
                let op: &str = match bin.op() {
                    koopa::ir::BinaryOp::Sub => "sub",
                    koopa::ir::BinaryOp::Eq => "eq",
                    _ => unreachable!(),
                };
                let value = self.value_to_string(func, value);
                writeln!(&mut self.output, "{} = {} {}, {}", value, op, lhs, rhs).unwrap();
            }
            _ => writeln!(&mut self.output, "{:?}", data.kind()).unwrap(),
        }
    }

    fn value_to_string(&self, func: &FunctionData, value: Value) -> String {
        let data = func.dfg().value(value);
        match data.kind() {
            ValueKind::Integer(int) => int.value().to_string(),
            _ => {
                let name = data.name().as_ref().unwrap();
                format!("{}", name)
            }
        }
    }

    fn indent(&mut self) {
        for _ in 0..self.indent_level {
            self.output.push_str("  ");
        }
    }
}
