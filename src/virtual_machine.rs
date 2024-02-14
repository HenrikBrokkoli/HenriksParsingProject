/*use std::collections::HashMap;
use std::{fmt, mem};
use crate::errors::ParserError;
use crate::parse_funcs::{parse_symbol, parse_whitespace,parse_var_name};
use crate::peekables::{ParseProcess, TPeekable};
use crate::virtual_machine::Arg::{VSInt, VUInt};





#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum Arg {
    VUInt(usize),
    VSInt(isize),
}

pub struct Operbator {
    op: Op,
    arguments: Vec<Arg>,
}

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum OpNarg {
    Add,
    Sub,
    Mul,
    Div,
    Pop,
    Push,
    Swap,
    Dup,
    Jump,
    Label,
    JumpZer,
    JumpNeg,
    JumpPos,
    JumpNegZer,
    JumpPosZer,
    Call,
    Ret,
    Set,
    Load,
    LoadTS,
    SetFS,
}

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Pop,
    Push(isize),
    Swap,
    Dup,
    Jump(usize),
    Label(usize),
    JumpZer(usize),
    JumpNeg(usize),
    JumpPos(usize),
    JumpNegZer(usize),
    JumpPosZer(usize),
    Call(usize),
    Ret,
    Set(usize, isize),
    Load(usize),
    LoadTS(usize),
    SetFS(usize),
}

impl fmt::Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

pub struct StackFrame {
    return_adress: usize,
    variables: HashMap<usize, isize>,
}


pub struct VirtualMachine {
    stack: Vec<isize>,
    code: Vec<Op>,
    instruction_pointer: usize,
    io_dest: Box<dyn FnMut(isize)>,
    return_stack: Vec<StackFrame>,
    labels: HashMap<usize, usize>,
    variables: HashMap<usize, isize>,
}

impl VirtualMachine {
    pub fn new(io_dest: Box<dyn FnMut(isize)>) -> VirtualMachine {
        VirtualMachine { stack: vec![], code: vec![], instruction_pointer: 0, io_dest, return_stack: vec![], labels: HashMap::new(), variables: HashMap::new() }
    }

    pub fn initialize_labels(&mut self) {
        for (line, op) in self.code.iter().enumerate() {
            if let Op::Label(label) = op {
                self.labels.insert(*label, line);
            }
        }
    }

    pub fn run_line(&mut self) -> Result<(), String> {
        let op = *(self.code.get(self.instruction_pointer).ok_or("CodePointer außerhalb von Code")?);
        self.eval_operator(op)?;
        self.instruction_pointer += 1;

        Ok(())
    }

    pub fn load_code(&mut self, code: Vec<Op>) {
        self.code = code;
        self.initialize_labels()
    }

    pub fn run(&mut self) -> Result<(), String> {
        while self.instruction_pointer < self.code.len() {
            self.run_line()?;
        }
        Ok(())
    }

    pub fn stp(&mut self) -> Result<isize, String> {
        self.stack.pop().ok_or(String::from("kein element im stack"))
    }

    fn st_op_two_args(&mut self, op: impl Fn(isize, isize) -> isize) -> Result<(), String> {
        let a = self.stp()?;
        let b = self.stp()?;
        self.stack.push(op(a, b));
        Ok(())
    }
    fn jump(&mut self, to: usize) -> Result<(), String> {
        let line = self.labels.get(&to).ok_or("label existiert nicht")?;
        self.instruction_pointer = *line;
        Ok(())
    }

    fn call(&mut self, to: usize) -> Result<(), String> {
        let var = mem::take(&mut self.variables);
        self.return_stack.push(StackFrame { return_adress: self.instruction_pointer, variables: var });
        self.jump(to)?;
        Ok(())
    }

    fn ret(&mut self) -> Result<(), String> {
        let stack_frame = self.return_stack.pop().ok_or("return stack is empty")?;
        self.instruction_pointer = stack_frame.return_adress;
        self.variables = stack_frame.variables;
        Ok(())
    }

    fn jump_if(&mut self, to: usize, cond: fn(isize) -> bool) -> Result<(), String> {
        if cond(self.stp()?) {
            self.jump(to)?;
        }
        Ok(())
    }
    fn swap(&mut self) -> Result<(), String> {
        let a = self.stp()?;
        let b = self.stp()?;
        self.stack.push(a);
        self.stack.push(b);
        Ok(())
    }

    pub fn set(&mut self, name: usize, value: isize) {
        self.variables.insert(name, value);
    }

    pub fn set_from_stack(&mut self, name: usize) -> Result<(), String> {
        let val = self.stp()?;
        self.set(name, val);
        Ok(())
    }
    pub fn load(&mut self, name: usize) -> Result<isize, String> {
        Ok(*self.variables.get(&name).ok_or("Variable existiert nicht")?)
    }
    pub fn load_to_stack(&mut self, name: usize) -> Result<(), String> {
        let val = self.load(name)?;
        self.stack.push(val);
        Ok(())
    }

    pub fn load_to_io(&mut self, name: usize) -> Result<(), String> {
        let val = self.load(name)?;
        (self.io_dest)(val);
        Ok(())
    }

    pub fn eval_operator(&mut self, op: Op) -> Result<(), String> {
         match op {
             Op::Add => { self.st_op_two_args(|a, b| -> isize{ a + b })? }
             Op::Sub => { self.st_op_two_args(|a, b| -> isize{ b - a })? }
             Op::Mul => { self.st_op_two_args(|a, b| -> isize{ a * b })? }
             Op::Div => { self.st_op_two_args(|a, b| -> isize{ b / a })? }
             Op::Swap => {
                 self.swap()?;
             }
             Op::Pop => {
                 let val = self.stp()?;
                 (self.io_dest)(val)
             }
             Op::Push(x) => {
                 self.stack.push(x)
             }
             Op::Dup => {
                 let a = *self.stack.last().ok_or("kein element im Stack")?;
                 self.stack.push(a)
             }
             Op::Label(_) => (),
             Op::Jump(label) => {
                 self.jump(label)?;
             }
             Op::JumpZer(label) => {
                 self.jump_if(label, |x| x == 0)?;
             }
             Op::JumpNeg(label) => {
                 self.jump_if(label, |x| x < 0)?;
             }
             Op::JumpPos(label) => {
                 self.jump_if(label, |x| x > 0)?;
             }
             Op::JumpNegZer(label) => {
                 self.jump_if(label, |x| x <= 0)?;
             }
             Op::JumpPosZer(label) => {
                 self.jump_if(label, |x| x >= 0)?;
             }
             Op::Call(label) => {
                 self.call(label)?;
             }
             Op::Ret => {
                 self.ret()?;
             }
             Op::Set(name, value) => { self.set(name, value) }
             Op::Load(name) => { self.load_to_io(name)? }
             Op::SetFS(name) => { self.set_from_stack(name)? }
             Op::LoadTS(name) => { self.load_to_stack(name)? }
         };
         Ok(())
    }

    pub fn parse_lines<T>(&mut self, frame: &mut ParseProcess<T>) -> Result<Vec<OperatorFunc>, ParserError> where T: TPeekable<Item=char> {
        parse_whitespace(frame);
        let mut ops = vec![self.parse_op_code_real(frame)?];
        while parse_symbol(frame, ';').is_ok() {
            ops.push(self.parse_op_code_real(frame)?);
        }
        Ok(ops)
    }


    pub fn parse_op_code_real<T>(&mut self, frame: &mut ParseProcess<T>) -> Result<OperatorFunc, ParserError> where T: TPeekable<Item=char> {
        let name = parse_var_name(frame)?;
        parse_symbol(frame, '(')?;

        let res: for<'r> fn(&'r Vec<Arg>) -> Result<Op, String> = match name.as_str() {
            "Add" => {
                |_| Ok(Op::Add)
            }
            "Sub" => {
                |_| Ok(Op::Sub)
            }
            "Mul" => {
                |_| Ok(Op::Mul)
            }
            "Div" => {
                |_| Ok(Op::Div)
            }
            "Pop" => {
                |_| Ok(Op::Pop)
            }
            "Push" => {
                |x| {
                    Ok(Op::Push(arg_to_isize(arg_get(x, 0)?)?))
                }
            }
            "Swap" => {
                |_| Ok(Op::Swap)
            }
            "Dup" => {
                |_| Ok(Op::Dup)
            }
            "Jump" => {
                |args| {
                    Ok(Op::Jump(arg_to_usize(arg_get(args, 0)?)?))
                }
            }
            "Label" => {
                |args| {
                    Ok(Op::Label(arg_to_usize(arg_get(args, 0)?)?))
                }
            }
            "JumpZer" => {
                |args| {
                    Ok(Op::JumpZer(arg_to_usize(arg_get(args, 0)?)?))
                }
            }
            "JumpNeg" => {
                |args| {
                    Ok(Op::JumpNeg(arg_to_usize(arg_get(args, 0)?)?))
                }
            }
            "JumpPos" => {
                |args| {
                    Ok(Op::JumpPos(arg_to_usize(arg_get(args, 0)?)?))
                }
            }
            "JumpNegZer" => {
                |args| {
                    Ok(Op::JumpNegZer(arg_to_usize(arg_get(args, 0)?)?))
                }
            }
            "JumpPosZer" => {
                |args| {
                    Ok(Op::JumpPosZer(arg_to_usize(arg_get(args, 0)?)?))
                }
            }
            "Call" => {
                |args| {
                    Ok(Op::Call(arg_to_usize(arg_get(args, 0)?)?))
                }
            }
            "Ret" => {
                |_| Ok(Op::Ret)
            }
            "Set" => {
                |args| {
                    let first = arg_to_usize(arg_get(args, 0)?)?;
                    let second = arg_to_isize(arg_get(args, 1)?)?;
                    Ok(Op::Set(first, second))
                }
            }
            "Load" => {
                |args| {
                    Ok(Op::Load(arg_to_usize(arg_get(args, 0)?)?))
                }
            }
            "LoadTS" => {
                |args| {
                    Ok(Op::LoadTS(arg_to_usize(arg_get(args, 0)?)?))
                }
            }
            "SetFS" => {
                |args| {
                    Ok(Op::SetFS(arg_to_usize(arg_get(args, 0)?)?))
                }
            }
            _ => { return Err(ParserError::Impossible); }
        };
        Ok(res)
    }
}


pub fn arg_get(args: &[Arg], index: usize) -> Result<Arg, String> {
    Ok(*(args.get(index).ok_or(format!("No Argument at index {index}", index = index)))?)
}

pub fn arg_to_usize(arg: Arg) -> Result<usize, String> {
    if let VUInt(x) = arg {
        Ok(x)
    } else { Err("Argument is not usize".to_string()) }
}

pub fn arg_to_isize(arg: Arg) -> Result<isize, String> {
    if let VSInt(x) = arg {
        Ok(x)
    } else { Err("Argument is not isize".to_string()) }
}


#[cfg(test)]
mod tests {
    use crate::virtual_machine::{Op, VirtualMachine};
    use std::sync::{Arc, Mutex};
    use crate::virtual_machine::Op::{Call, Label};

    #[test]
    fn test_simple_add() {
        let code = vec![Op::Push(1), Op::Push(2), Op::Add, Op::Pop];
        let io = Arc::new(Mutex::new(vec![]));
        let io_clone = Arc::clone(&io);
        let io_closure = Box::new(move |x: isize| io_clone.lock().unwrap().push(x));
        let mut virtual_machine = VirtualMachine::new(io_closure);
        virtual_machine.load_code(code);
        virtual_machine.run().unwrap();
        assert_eq!(3, io.lock().unwrap()[0]);
    }

    #[test]
    fn test_simple_add1() {
        let code = vec![Op::Push(1), Op::Dup, Op::Add, Op::Pop];
        let io = Arc::new(Mutex::new(vec![]));
        let io_clone = Arc::clone(&io);

        let io_closure = Box::new(move |x: isize| io_clone.lock().unwrap().push(x));
        let mut virtual_machine = VirtualMachine::new(io_closure);
        virtual_machine.load_code(code);
        virtual_machine.run().unwrap();
        assert_eq!(2, io.lock().unwrap()[0]);
    }

    #[test]
    fn test_simple_jump() {
        let code = vec![Op::Push(1), Op::Jump(1), Op::Push(100), Op::Label(1), Op::Pop];
        let io = Arc::new(Mutex::new(vec![]));
        let io_clone = Arc::clone(&io);

        let io_closure = Box::new(move |x: isize| io_clone.lock().unwrap().push(x));
        let mut virtual_machine = VirtualMachine::new(io_closure);
        virtual_machine.load_code(code);
        virtual_machine.run().unwrap();
        assert_eq!(1, io.lock().unwrap()[0]);
    }


    #[test]
    fn test_cond_jump() {
        let code = vec![Op::Push(3), Label(1), Op::Push(1), Op::Sub, Op::Dup, Op::JumpPos(1), Op::Pop];
        let io = Arc::new(Mutex::new(vec![]));
        let io_clone = Arc::clone(&io);

        let io_closure = Box::new(move |x: isize| io_clone.lock().unwrap().push(x));
        let mut virtual_machine = VirtualMachine::new(io_closure);
        virtual_machine.load_code(code);
        virtual_machine.run().unwrap();
        assert_eq!(0, io.lock().unwrap()[0]);
    }

    #[test]
    fn test_call() {
        let code = vec![Op::Jump(1), Label(2), Op::Add, Op::Add, Op::Ret, Op::Label(1), Op::Push(1), Op::Dup, Op::Dup, Call(2), Op::Pop];
        let io = Arc::new(Mutex::new(vec![]));
        let io_clone = Arc::clone(&io);

        let io_closure = Box::new(move |x: isize| io_clone.lock().unwrap().push(x));
        let mut virtual_machine = VirtualMachine::new(io_closure);
        virtual_machine.load_code(code);
        virtual_machine.run().unwrap();
        assert_eq!(3, io.lock().unwrap()[0]);
    }

    #[test]
    fn test_call_local_vars() {
        let code = vec![Op::Jump(1), Label(2), Op::Set(1, 1), Op::Load(1), Op::Ret, Op::Label(1), Op::Set(1, 2), Op::Load(1), Call(2), Op::Load(1)];
        let io = Arc::new(Mutex::new(vec![]));
        let io_clone = Arc::clone(&io);

        let io_closure = Box::new(move |x: isize| io_clone.lock().unwrap().push(x));
        let mut virtual_machine = VirtualMachine::new(io_closure);
        virtual_machine.load_code(code);
        virtual_machine.run().unwrap();
        assert_eq!(2, io.lock().unwrap()[0]);
        assert_eq!(1, io.lock().unwrap()[1]);
        assert_eq!(2, io.lock().unwrap()[2]);
    }
}
*/