use std::collections::{HashMap, HashSet};

use koopa::ir::{layout::BasicBlockNode, BasicBlock, FunctionData, Value, ValueKind};

// #[derive(Default)]
// struct LivenessInfo {
//     live_in: HashMap<Value, HashSet<Value>>,
//     live_out: HashMap<Value, HashSet<Value>>,
// }

// #[derive(Default)]
// struct InterferenceEdge {
//     edges: HashMap<Value, HashSet<Value>>,
//     degree: HashMap<Value, usize>,
// }
// impl InterferenceEdge {
//     fn new() -> Self {
//         Self {
//             edges: HashMap::new(),
//             degree: HashMap::new(),
//         }
//     }

//     fn add_edge(&mut self, v1: Value, v2: Value) {
//         self.edges.entry(v1).or_default().insert(v2);
//         self.edges.entry(v2).or_default().insert(v1);
//         *self.degree.entry(v1).or_default() += 1;
//         *self.degree.entry(v2).or_default() += 1;
//     }

//     fn neighbors(&self, value: &Value) -> impl Iterator<Item = &Value> {
//         self.edges
//             .get(value)
//             .map(|set| set.iter())
//             .into_iter()
//             .flatten()
//     }

//     fn get_degree(&self, value: &Value) -> usize {
//         self.degree.get(value).copied().unwrap_or(0)
//     }
// }
#[derive(Default)]
pub struct RiscvRegisterManager {
    temp_regs: [bool; 7],   // t0-t6
    saved_regs: [bool; 12], // s0-s11
    arg_regs: [bool; 8],    // a0-a7
    pub(crate) value_reg_map: HashMap<Value, String>,
    pub(crate) value_use_count: HashMap<Value, usize>,
    pub(crate) stack_slots: HashMap<Value, i32>,
    pub(crate) current_stack_offset: i32,
    // interference_edge: InterferenceEdge,
}

impl RiscvRegisterManager {
    fn is_value_dead(&self, val: &Value) -> bool {
        self.value_use_count
            .get(val)
            .map_or(true, |&count| count == 0)
    }
    pub fn reset_registers(&mut self) {
        self.value_reg_map.clear();
        self.value_use_count.clear();
        self.temp_regs = [false; 7];
        self.saved_regs = [false; 12];
        self.arg_regs = [false; 8];
    }

    pub fn reset_stack(&mut self) {
        self.stack_slots.clear();
        self.current_stack_offset = 0;
    }

    pub fn new() -> Self {
        Self {
            temp_regs: [false; 7],
            saved_regs: [false; 12],
            arg_regs: [false; 8],
            value_reg_map: HashMap::new(),
            value_use_count: HashMap::new(),
            stack_slots: HashMap::new(),
            current_stack_offset: 0,
            // interference_edge: InterferenceEdge::new(),
        }
    }

    pub(crate) fn allocate_tmp(&mut self) -> Option<String> {
        for (i, used) in self.temp_regs.iter_mut().enumerate() {
            if !*used {
                *used = true;
                return Some(format!("t{}", i));
            }
        }
        // spill to stack
        for reg in &["t0", "t1", "t2", "t3", "t4", "t5", "t6"] {
            if let Some((val, _)) = self
                .value_reg_map
                .iter()
                .find(|(_, r)| r == reg)
                .and_then(|(v, r)| Some((*v, r.clone())))
            {
                // 溢出到栈
                let offset = self.spill_to_stack(val);
                self.stack_slots.insert(val, offset);
                self.free_register(reg, val);
                return Some(reg.to_string());
            }
        }
        None
    }
    pub(crate) fn allocate_saved(&mut self) -> Option<String> {
        for (i, used) in self.saved_regs.iter_mut().enumerate() {
            if !*used {
                *used = true;
                return Some(format!("s{}", i));
            }
        }
        None
    }

    pub(crate) fn free_register(&mut self, reg: &str, val: Value) {
        if self.is_value_dead(&val) {
            let reg_type = reg.chars().next().unwrap();
            let reg_num: usize = reg[1..].parse().unwrap();
            match reg_type {
                't' => self.temp_regs[reg_num] = false,
                's' => self.saved_regs[reg_num] = false,
                'a' => self.arg_regs[reg_num] = false,
                _ => {}
            }
        }
    }

    pub fn try_reuse_stack_slot(&mut self) -> Option<i32> {
        for (&val, &offset) in &self.stack_slots {
            if self.is_value_dead(&val) {
                return Some(offset);
            }
        }
        None
    }

    pub fn spill_to_stack(&mut self, val: Value) -> i32 {
        if let Some(offset) = self.try_reuse_stack_slot() {
            self.stack_slots.insert(val, offset);
            return offset;
        }
        let offset = self.current_stack_offset;
        self.stack_slots.insert(val, offset);
        self.current_stack_offset += 4;
        offset
    }
    pub fn get_calling_convention_reg(&self, index: usize) -> Option<String> {
        if index < 8 {
            Some(format!("a{}", index))
        } else {
            None
        }
    }

    pub fn reserve_for_call(&mut self, arg_count: usize) {
        for i in 0..arg_count.min(8) {
            self.arg_regs[i] = true;
        }
    }
    pub fn get_return_reg(&self) -> String {
        "a0".to_string()
    }

    pub fn mark_register_live(&mut self, reg: &str) {
        let reg_type = reg.chars().next().unwrap();
        let reg_num: usize = reg[1..].parse().unwrap();

        match reg_type {
            't' => self.temp_regs[reg_num] = true,
            's' => self.saved_regs[reg_num] = true,
            'a' => self.arg_regs[reg_num] = true,
            _ => {}
        }
    }
    pub fn decrease_use_count(&mut self, val: &Value) {
        if let Some(count) = self.value_use_count.get_mut(val) {
            if *count > 0 {
                *count -= 1;
            }
        }
    }

    pub(crate) fn after_value_use(&mut self, val: Value) {
        self.decrease_use_count(&val);
        if self.is_value_dead(&val) {
            if let Some(reg) = self.value_reg_map.get(&val).cloned() {
                self.free_register(&reg, val);
            }
        }
    }
    pub(crate) fn should_allocate_register(&self, val: Value) -> bool {
        if let Some(&use_count) = self.value_use_count.get(&val) {
            use_count > 1
        } else {
            false
        }
    }

    pub fn generate_prologue(&self) -> Vec<String> {
        let mut prologue = Vec::new();
        let stack_size = self.current_stack_offset;
        let aligned_size = (stack_size + 15) / 16 * 16;
        if aligned_size > 0 {
            if aligned_size <= 2047 {
                prologue.push(format!("  addi sp, sp, -{}", aligned_size));
            } else {
                prologue.push(format!("  li t0 , {}", aligned_size));
                prologue.push("  sub sp, sp, t0".to_string());
            }
        }

        let mut to_save = Vec::new();

        for (i, &used) in self.saved_regs.iter().enumerate() {
            if used {
                to_save.push(format!("s{}", i));
            }
        }
        if to_save.len() > 0 {
            prologue.push(format!("  addi sp, sp, -{}", (to_save.len() * 4) as i32));
            for (i, reg) in to_save.iter().enumerate() {
                prologue.push(format!("  sw {}, {}(sp)", reg, i * 4));
            }
        }
        prologue
    }

    pub fn generate_epilogue(&self) -> Vec<String> {
        let mut epilogue = Vec::new();
        let mut aligned_size = self.current_stack_offset;
        if aligned_size > 0 {
            if aligned_size <= 2047 {
                epilogue.push(format!("  addi sp, sp, {}", aligned_size));
            } else {
                epilogue.push(format!("  li t0 , {}", aligned_size));
                epilogue.push("  add sp, sp, t0".to_string());
            }
        }
        let mut to_restore = Vec::new();

        for (i, &used) in self.saved_regs.iter().enumerate() {
            if used {
                to_restore.push(format!("s{}", i));
            }
        }
        if to_restore.len() > 0 {
            for (i, reg) in to_restore.iter().enumerate() {
                epilogue.push(format!("  lw {}, {}(sp)", reg, i * 4));
            }
            epilogue.push(format!("  addi sp, sp, {}", (to_restore.len() * 4) as i32));
        }
        epilogue
    }
}

// impl RiscvRegisterManager {
//     fn build_interference_graph(
//         &self,
//         func: &FunctionData,
//         live_vars: &HashMap<Value, HashSet<Value>>,
//     ) -> HashMap<Value, HashSet<Value>> {
//         let mut graph = HashMap::new();

//         for (bb, _) in func.layout().bbs() {
//             let bb_node = func.layout().bbs().node(bb).unwrap();

//             for (inst, _) in bb_node.insts() {
//                 if let Some(live_set) = live_vars.get(inst) {
//                     for &live_val in live_set {
//                         graph
//                             .entry(*inst)
//                             .or_insert_with(HashSet::new)
//                             .insert(live_val);
//                         graph
//                             .entry(live_val)
//                             .or_insert_with(HashSet::new)
//                             .insert(*inst);
//                     }
//                 }
//             }
//         }

//         graph
//     }

//     fn compute_block_liveness(
//         &self,
//         func: &FunctionData,
//     ) -> (
//         HashMap<BasicBlock, HashSet<Value>>,
//         HashMap<BasicBlock, HashSet<Value>>,
//     ) {
//         let mut bb_live_in = HashMap::new();
//         let mut bb_live_out = HashMap::new();

//         // 初始化每个基本块的 live_in 和 live_out 集合
//         for (bb, _) in func.layout().bbs() {
//             bb_live_in.insert(*bb, HashSet::new());
//             bb_live_out.insert(*bb, HashSet::new());
//         }

//         let mut changed = true;
//         while changed {
//             changed = false;

//             for (bb, _) in func.layout().bbs() {
//                 let bb_node = func.layout().bbs().node(*bb).unwrap();
//                 let old_in = bb_live_in[bb].clone();
//                 let old_out = bb_live_out[bb].clone();

//                 // 获取当前块的 def 和 use 集合
//                 let (bb_def, bb_use) = self.compute_block_def_use(bb_node);

//                 // 更新 out 集合：遍历当前块的最后一条指令找到后继块
//                 let mut new_out = HashSet::new();
//                 if let Some((last_inst, _)) = bb_node.insts().next_back() {
//                     let inst_data = func.dfg().value(*last_inst);
//                     match inst_data.kind() {
//                         ValueKind::Branch(br) => {
//                             // 条件跳转有两个后继
//                             new_out.extend(bb_live_in[&br.true_bb()].iter().cloned());
//                             new_out.extend(bb_live_in[&br.false_bb()].iter().cloned());
//                         }
//                         ValueKind::Jump(jump) => {
//                             // 无条件跳转只有一个后继
//                             new_out.extend(bb_live_in[&jump.target()].iter().cloned());
//                         }
//                         ValueKind::Return(_) => {
//                             // return 指令没有后继
//                         }
//                         _ => {}
//                     }
//                 }
//                 bb_live_out.insert(*bb, new_out.clone());

//                 // 更新 in 集合: in[B] = use[B] ∪ (out[B] - def[B])
//                 let mut new_in = bb_use; // 首先加入所有 use 的变量
//                 for &val in &new_out {
//                     // 然后加入 out 中不在 def 里的变量
//                     if !bb_def.contains(&val) {
//                         new_in.insert(val);
//                     }
//                 }
//                 bb_live_in.insert(*bb, new_in);

//                 // 检查是否发生变化
//                 if bb_live_in[bb] != old_in || bb_live_out[bb] != old_out {
//                     changed = true;
//                 }
//             }
//         }

//         (bb_live_in, bb_live_out)
//     }
//     fn compute_block_def_use(&self, bb_node: &BasicBlockNode) -> (HashSet<Value>, HashSet<Value>) {
//         let mut def_set = HashSet::new();
//         let mut use_set = HashSet::new();

//         // 从前向后遍历指令收集 def 和 use
//         for (inst, _) in bb_node.insts() {
//             let data = bb_node.dfg().value(*inst);

//             // 每条指令的结果都是一个 def
//             def_set.insert(*inst);

//             // 根据指令类型收集 use
//             match data.kind() {
//                 ValueKind::Binary(binary) => {
//                     use_set.insert(binary.lhs());
//                     use_set.insert(binary.rhs());
//                 }
//                 ValueKind::Branch(br) => {
//                     use_set.insert(br.cond());
//                 }
//                 ValueKind::Call(call) => {
//                     use_set.extend(call.args());
//                 }
//                 ValueKind::Return(ret) => {
//                     if let Some(val) = ret.value() {
//                         use_set.insert(val);
//                     }
//                 }
//                 // 添加其他可能的指令类型...
//                 _ => {}
//             }
//         }

//         (def_set, use_set)
//     }
//     fn analyze_liveness(
//         &self,
//         func: &FunctionData,
//         bb_live_in: &HashMap<BasicBlock, HashSet<Value>>,
//         bb_live_out: &HashMap<BasicBlock, HashSet<Value>>,
//     ) -> HashMap<Value, HashSet<Value>> {
//         let mut live_vars = HashMap::new();

//         for (bb, _) in func.layout().bbs() {
//             let bb_node = func.layout().bbs().node(bb).unwrap();
//             let mut current_live = bb_live_out[&bb].clone();

//             // 从后向前遍历指令
//             let mut insts = vec![];
//             for (inst, _) in bb_node.insts() {
//                 insts.push((inst, func.dfg().value(*inst)));
//             }
//             for (inst, _) in insts.iter().rev() {
//                 let data = func.dfg().value(**inst);

//                 // 记录当前指令的活跃变量集合
//                 live_vars.insert(**inst, current_live.clone());

//                 // 先移除定值
//                 current_live.remove(inst);

//                 // 添加使用的值
//                 match data.kind() {
//                     ValueKind::Binary(binary) => {
//                         current_live.insert(binary.lhs());
//                         current_live.insert(binary.rhs());
//                     }
//                     ValueKind::Call(call) => {
//                         for &arg in call.args() {
//                             current_live.insert(arg);
//                         }
//                     }
//                     ValueKind::Return(ret) => {
//                         if let Some(val) = ret.value() {
//                             current_live.insert(val);
//                         }
//                     }
//                     // TODO: 处理其他指令类型
//                     _ => {}
//                 }
//             }
//         }
//         live_vars
//     }

//     fn simplify_phase(
//         &mut self,
//         interference_graph: &HashMap<Value, HashSet<Value>>,
//     ) -> (Vec<Value>, HashMap<Value, String>) {
//         let mut select_stack = Vec::new();
//         let mut color_map = HashMap::new();
//         let mut spilled_nodes = HashSet::new();
//         let mut working_graph = interference_graph.clone();
//         let k = 7; // 可用的临时寄存器数量 (t0-t6)

//         // 简化阶段：重复移除低度数节点直到图为空
//         loop {
//             let mut found_low_degree = false;

//             // 查找所有度数 < k 的节点
//             let low_degree_nodes: Vec<_> = working_graph
//                 .iter()
//                 .filter(|(&value, neighbors)| {
//                     !spilled_nodes.contains(&value) && neighbors.len() < k
//                 })
//                 .map(|(&value, _)| value)
//                 .collect();
//             for value in low_degree_nodes {
//                 select_stack.push(value);
//                 spilled_nodes.insert(value);
//                 found_low_degree = true;
//             }
//             if !found_low_degree {
//                 if let Some(&value) = working_graph.keys().find(|&&v| !spilled_nodes.contains(&v)) {
//                     let offset = self.spill_to_stack(value);
//                     color_map.insert(value, format!("{}(sp)", offset));
//                     spilled_nodes.insert(value);
//                     select_stack.push(value);
//                 } else {
//                     break;
//                 }
//             }

//             working_graph.retain(|k, _| !spilled_nodes.contains(k));
//             for edges in working_graph.values_mut() {
//                 edges.retain(|v| !spilled_nodes.contains(v));
//             }
//         }

//         (select_stack, color_map)
//     }

//     fn select_phase(
//         &mut self,
//         interference_graph: &HashMap<Value, HashSet<Value>>,
//         mut select_stack: Vec<Value>,
//         color_map: &mut HashMap<Value, String>,
//     ) {
//         let available_regs = vec!["t0", "t1", "t2", "t3", "t4", "t5", "t6"];

//         while let Some(value) = select_stack.pop() {
//             // 如果节点已经被标记为溢出，跳过
//             if color_map.contains_key(&value) {
//                 continue;
//             }

//             let mut used_colors = HashSet::new();

//             // 收集邻接点使用的颜色
//             if let Some(neighbors) = interference_graph.get(&value) {
//                 for &neighbor in neighbors {
//                     if let Some(color) = color_map.get(&neighbor) {
//                         used_colors.insert(color);
//                     }
//                 }
//             }

//             // 选择一个可用的颜色
//             if let Some(&reg) = available_regs
//                 .iter()
//                 .find(|&&reg| !used_colors.contains(&reg.to_string()))
//             {
//                 color_map.insert(value, reg.to_string());
//                 self.mark_register_live(reg);
//             } else {
//                 // 理论上不应该到这里，因为我们在简化阶段已经处理了所有可能的溢出
//                 let offset = self.spill_to_stack(value);
//                 color_map.insert(value, format!("{}(sp)", offset));
//             }
//         }
//     }
// }

// impl RiscvRegisterManager {
//     pub fn allocate_registers_graph_coloring(&mut self, func: &FunctionData) {
//         let (bb_live_in, bb_live_out) = self.compute_block_liveness(func);
//         let live_vars = self.analyze_liveness(func, &bb_live_in, &bb_live_out);

//         let interference_graph = self.build_interference_graph(func, &live_vars);

//         let (select_stack, mut color_map) = self.simplify_phase(&interference_graph);

//         self.select_phase(&interference_graph, select_stack, &mut color_map);

//         self.value_reg_map = color_map;
//     }
// }
