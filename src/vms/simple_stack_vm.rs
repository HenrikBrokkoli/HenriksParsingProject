use crate::errors::ParserError;
use crate::peekables::{ParseProcess, TPeekable};
use crate::tree::{NodeId, Tree};
use crate::vms::VM;

pub struct SimpleStackVmState {
    pub stack: Vec<usize>,
    pub reg: usize,
    pub reg2: usize,
    pub error: usize,
}

pub enum Instruction {
    Add,
    RegAddConst(usize),
    Sub,
    Digit,
    PopToReg,
    PopToReg2,
    PopDiscard,
    PushConst(usize),
    PushFromTree,
    PushReg,
    PushReg2,
    PrintReg,
    PrintReg2,
    PrintError,
    Pow(usize)
}

pub struct SimpleStackVm {}

impl VM for SimpleStackVm {
    type Tstate = SimpleStackVmState;
    type Tinstrution = Instruction;
    fn parse_instructions<T>(
        &self,
        prod_name: &str,
        to_parse: &mut ParseProcess<T>,
    ) -> Result<Vec<Self::Tinstrution>, ParserError>
    where
        T: TPeekable<Item = char>,
    {
        let mut c = to_parse.next();
        while let Some(_cc) = c {
            c = to_parse.next()
        }
        let instruction = match prod_name {
            "add" => vec![Instruction::PopToReg, Instruction::Add],
            "sub" => vec![Instruction::PopToReg, Instruction::Sub],
            "number" => vec![Instruction::PopDiscard],
            "digit" => vec![Instruction::PushFromTree,Instruction::PushConst(1)],
            "number_s_" => vec![Instruction::PopToReg, Instruction::PopToReg2, Instruction::PopDiscard, Instruction::Pow(10),Instruction::RegAddConst(1),Instruction::PushReg],
            "print" => vec![Instruction::PopToReg, Instruction::PrintReg],
            _ =>  vec![],
        };
        Ok(instruction)
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
                    let res =  var - state.reg;
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
                    if let Ok(value) = node.data.parse::<usize>() {
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
            },
            Instruction::PrintReg => {
                let reg= state.reg;
                println!("{reg}");
            }
            Instruction::PrintReg2 => {
                let reg= state.reg2;
                println!("{reg}");
            }
            Instruction::PrintError => {
                let reg= state.error;
                println!("{reg}");
            }
            Instruction::Pow(base) => {
                if let Some(val)=state.stack.pop(){
                    let res = val * base.pow(state.reg as u32) + state.reg2;
                    state.stack.push(res);
                }else { state.error = 1 }
            }
            Instruction::PushReg => {state.stack.push(state.reg)}
            Instruction::PushReg2 => {state.stack.push(state.reg2)}
            Instruction::RegAddConst(c) => {
                state.reg= state.reg + c;
            }
        }
    }

    fn create_new_state() -> Self::Tstate {
        SimpleStackVmState { stack: vec![],reg:0,reg2:0,error:0 }
    }
}
