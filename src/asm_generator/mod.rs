mod riscv_asm_generator;

use std::{collections::HashMap, fmt::Write};

use koopa::ir::{BinaryOp, FunctionData, Program, Value, ValueKind};

pub struct AsmGenerator {
    next_reg: usize,
    output: String,
    value_reg_map: HashMap<Value, String>,
    value_use_count: HashMap<Value, usize>,
}

impl AsmGenerator {
    pub fn new() -> Self {
        Self {
            next_reg: 0,
            output: String::new(),
            value_reg_map: HashMap::new(),
            value_use_count: HashMap::new(),
        }
    }

    fn count_value_uses(&mut self, func: &FunctionData) {
        for (bb, _) in func.layout().bbs() {
            for (inst, _) in func.layout().bbs().node(bb).unwrap().insts() {
                let data = func.dfg().value(*inst);
                match data.kind() {
                    ValueKind::Binary(binary) => {
                        *self.value_use_count.entry(binary.lhs()).or_insert(0) += 1;
                        *self.value_use_count.entry(binary.rhs()).or_insert(0) += 1;
                    }
                    ValueKind::Return(ret) => {
                        if let Some(val) = ret.value() {
                            *self.value_use_count.entry(val).or_insert(0) += 1;
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    fn decrease_use_count(&mut self, val: Value) {
        if let Some(count) = self.value_use_count.get_mut(&val) {
            if *count > 0 {
                *count -= 1;
            }
        }
    }

    // get next temp register
    fn next_temp_reg(&mut self) -> String {
        let reg = format!("t{}", self.next_reg);
        self.next_reg = (self.next_reg + 1) % 7; // t0-t6
        reg
    }

    pub fn generate_program(&mut self, program: &Program) -> String {
        self.output.clear();

        writeln!(&mut self.output, ".text").unwrap();

        for func in program.func_layout() {
            self.generate_function(program.func(*func));
        }
        self.output.clone()
    }

    pub fn generate_function(&mut self, func: &FunctionData) {
        self.count_value_uses(func);
        let func_name = func.name().strip_prefix("@").unwrap();
        writeln!(&mut self.output, ".global {}", func_name).unwrap(); // .global function
        writeln!(&mut self.output, "{}:", func_name).unwrap(); // function
        for (bb, _) in func.layout().bbs() {
            for (inst, _) in func.layout().bbs().node(bb).unwrap().insts() {
                self.generate_instruction(func, *inst);
            }
        }
    }

    pub fn generate_instruction(&mut self, func: &FunctionData, val: Value) {
        let data = func.dfg().value(val);

        match data.kind() {
            ValueKind::Return(ret) => {
                if let Some(val) = ret.value() {
                    if let Some(reg) = self.value_reg_map.get(&val) {
                        writeln!(&mut self.output, "  mv a0, {}", reg).unwrap();
                    } else {
                        self.generate_value(func, val);
                    }
                }
                writeln!(&mut self.output, "ret").unwrap();
            }
            ValueKind::Binary(binary) => {
                let lhs = binary.lhs();
                // let rhs = binary.rhs();
                let lhs_reg = self.get_reg_for_value(func, binary.lhs());
                let rhs_reg = self.get_reg_for_value(func, binary.rhs());

                let can_reuse_lhs = self
                    .value_use_count
                    .get(&lhs)
                    .map_or(false, |&count| count <= 1);

                let dst_reg = if can_reuse_lhs && lhs_reg != "x0" {
                    lhs_reg.clone()
                } else {
                    self.next_temp_reg()
                };

                match binary.op() {
                    BinaryOp::Eq => {
                        writeln!(
                            &mut self.output,
                            "  xor {}, {}, {}",
                            dst_reg, lhs_reg, rhs_reg
                        )
                        .unwrap();
                        writeln!(&mut self.output, "  seqz {}, {}", dst_reg, dst_reg).unwrap();
                    }
                    BinaryOp::Sub => {
                        writeln!(self.output, "  sub {}, {}, {}", dst_reg, lhs_reg, rhs_reg)
                            .unwrap();
                    }
                    _ => {}
                }
                self.value_reg_map.insert(val, dst_reg.clone());
            }

            _ => {}
        }
    }
    pub fn generate_value(&mut self, func: &FunctionData, val: Value) {
        let data = func.dfg().value(val);

        match data.kind() {
            ValueKind::Integer(c) => {
                writeln!(&mut self.output, "  li a0, {}", c.value()).unwrap();
            }
            ValueKind::Binary(_) => {
                if let Some(reg) = self.value_reg_map.get(&val) {
                    writeln!(&mut self.output, "  mv a0, {}", reg).unwrap();
                }
            }
            _ => {
                unreachable!("Unsupported value kind");
            }
        }
    }

    pub fn get_reg_for_value(&mut self, func: &FunctionData, val: Value) -> String {
        if let Some(reg) = self.value_reg_map.get(&val) {
            return reg.clone();
        }
        let data = func.dfg().value(val);
        match data.kind() {
            ValueKind::Integer(c) => {
                if c.value() == 0 {
                    return "x0".to_string();
                }
                let reg = self.next_temp_reg();
                writeln!(&mut self.output, "  li {}, {}", reg, c.value()).unwrap();
                self.value_reg_map.insert(val, reg.clone());
                reg
            }
            ValueKind::Binary(_) => {
                self.generate_instruction(func, val);

                self.value_reg_map[&val].clone()
            }
            _ => {
                unreachable!(
                    "get_reg_for_value: ValueKind({:?}) not handled or should be generated earlier",
                    data.kind()
                );
            }
        }
    }
}
