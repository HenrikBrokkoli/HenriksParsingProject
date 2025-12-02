use crate::errors::ParserError;
use crate::parse_funcs::{parse_var_name, parse_whitespace};
use crate::peekables::{ParseProcess, TPeekable};
use crate::tree::{NodeId, Tree};
use crate::vms::VM;

use crate::parse_funcs::{parse_symbol, parse_usize};

pub enum Instruction {
    Add,
    RegAddConst(isize),
    Sub,
    Digit,
    PopToReg,
    PopToReg2,
    PopDiscard,
    PushConst(isize),
    PushFromTree,
    PushReg,
    PushReg2,
    PrintReg,
    PrintReg2,
    PrintError,
    Pow(usize),
}

pub struct StackVmState {
    pub stack: Vec<isize>,
    pub reg: isize,
    pub reg2: isize,
    pub error: usize,
    pub instruction_counter: usize,
    pub instructions: Vec<Instruction>,
}
pub struct StackVm {}

impl VM for StackVm {
    type Tstate = StackVmState;
    type Tinstrution = Instruction;

    fn parse_instructions<'a, T>(
        &'a self,
        _prod_name: &str,
        to_parse: &mut ParseProcess<T>,
    ) -> Result<Vec<Self::Tinstrution>, ParserError>
    where
        T: TPeekable<Item = char>,
    {
        let mut instructions = Vec::new();
        parse_whitespace(to_parse);

        while let Some(_) = to_parse.peek() {
            let res = parse_var_name(to_parse)?;
            //match all instruction names to instructions
            match res.as_str() {
                "Add" => instructions.push(Instruction::Add),
                "RegAddConst" => {
                    parse_whitespace(to_parse);
                    let val = parse_usize(to_parse)?;
                    instructions.push(Instruction::RegAddConst(val as isize));
                }
                "Sub" => instructions.push(Instruction::Sub),
                "Digit" => instructions.push(Instruction::Digit),
                "PopToReg" => instructions.push(Instruction::PopToReg),
                "PopToReg2" => instructions.push(Instruction::PopToReg2),
                "PopDiscard" => instructions.push(Instruction::PopDiscard),
                "PushConst" => {
                    parse_whitespace(to_parse);
                    let val = parse_usize(to_parse)?;
                    instructions.push(Instruction::PushConst(val as isize));
                }
                "PushFromTree" => instructions.push(Instruction::PushFromTree),
                "PushReg" => instructions.push(Instruction::PushReg),
                "PushReg2" => instructions.push(Instruction::PushReg2),
                "PrintReg" => instructions.push(Instruction::PrintReg),
                "PrintReg2" => instructions.push(Instruction::PrintReg2),
                "PrintError" => instructions.push(Instruction::PrintError),
                "Pow" => {
                    parse_whitespace(to_parse);
                    let val = parse_usize(to_parse)?;
                    instructions.push(Instruction::Pow(val));
                }
                _ => {} // Handle unknown instructions if necessary
            }
            parse_symbol(to_parse, ';')?;
            parse_whitespace(to_parse);
        }
        Ok(instructions)
    }

    fn execute_instruction(
        &self,
        tree: &mut Tree<String>,
        cur_node: NodeId,
        instruction: &Self::Tinstrution,
        state: &mut Self::Tstate,
    ) {
        match instruction {
            Instruction::Add => {
                if let Some(var) = state.stack.pop() {
                    let res = state.reg + var;
                    state.stack.push(res);
                } else {
                    state.error = 1
                }
            }
            Instruction::Sub => {
                if let Some(var) = state.stack.pop() {
                    let res = var - state.reg;
                    state.stack.push(res);
                } else {
                    state.error = 1
                }
            }

            Instruction::Digit => {}
            Instruction::PopToReg => {
                if let Some(res) = state.stack.pop() {
                    state.reg = res;
                } else {
                    state.error = 1
                }
            }
            Instruction::PopToReg2 => {
                if let Some(res) = state.stack.pop() {
                    state.reg2 = res;
                } else {
                    state.error = 1
                }
            }
            Instruction::PushConst(c) => {
                state.stack.push(*c);
            }
            Instruction::PushFromTree => {
                if let Some(node) = tree
                    .get_by_path_or_none(cur_node, vec![0].into_iter())
                    .ok()
                    .flatten()
                {
                    if let Ok(value) = node.data.parse::<isize>() {
                        state.stack.push(value);
                    } else {
                        state.error = 2; // Parsing error
                    }
                } else {
                    state.error = 1; // Node not found or other error
                }
            }
            Instruction::PopDiscard => {
                if let Some(res) = state.stack.pop() {
                    _ = res;
                } else {
                    state.error = 1
                }
            }
            Instruction::PrintReg => {
                let reg = state.reg;
                println!("{reg}");
            }
            Instruction::PrintReg2 => {
                let reg = state.reg2;
                println!("{reg}");
            }
            Instruction::PrintError => {
                let reg = state.error;
                println!("{reg}");
            }
            Instruction::Pow(base) => {
                if let Some(val) = state.stack.pop() {
                    let res = val * base.pow(state.reg as u32) as isize + state.reg2;
                    state.stack.push(res);
                } else {
                    state.error = 1
                }
            }
            Instruction::PushReg => state.stack.push(state.reg),
            Instruction::PushReg2 => state.stack.push(state.reg2),
            Instruction::RegAddConst(c) => {
                state.reg = state.reg + c;
            }
        }
    }

    fn create_new_state() -> Self::Tstate {
        StackVmState {
            stack: vec![],
            reg: 0,
            reg2: 0,
            error: 0,
            instruction_counter: 0,
            instructions: vec![],
        }
    }
}
