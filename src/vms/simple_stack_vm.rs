use peekables::{ParseProcess, TPeekable};
use simple_graph::Graph;
use tree::{NodeId, Tree};
use vms::{Instruction, VM};


pub struct SimpleStackVmState {
    pub stack: Vec<usize>,
}


pub struct SimpleStackVm {}

impl VM for SimpleStackVm {
    type Tstate = SimpleStackVmState;
    fn make_instruction<T>(&self, prod_name: &str, to_parse: &mut ParseProcess<T>)
                           -> Result<Box<Instruction<Self::Tstate>>, String> where T: TPeekable<Item=char> {
        //TODO: to_parse is not handled right. Works for the example but will not work when user puts something in curly braces
        let mut c = to_parse.next();
        while let Some(_cc) = c {
            c = to_parse.next()
        }
        let instruction = match prod_name {
            "add" => |tree: &mut Tree<String>,cur_node:NodeId, state: &mut Self::Tstate| {
                let res = state.stack.pop().unwrap() + state.stack.pop().unwrap();
                state.stack.push(res);

                Ok(())
            },
            "sub" => |tree: &mut Tree<String>,cur_node:NodeId, state: &mut Self::Tstate| {
                let second = state.stack.pop().unwrap();
                let res = state.stack.pop().unwrap() - second;
                state.stack.push(res);
                Ok(())
            },
            "number" =>  |tree: &mut Tree<String>,cur_node:NodeId, state: &mut Self::Tstate| {
                let e= state.stack.pop().unwrap();
                Ok(())
            },
            "digit" =>  |tree: &mut Tree<String>,cur_node:NodeId, state: &mut Self::Tstate| {
                let digit_string= tree.get_by_path_or_none(cur_node, vec![0].into_iter()).unwrap().unwrap();
                state.stack.push(digit_string.data.parse::<usize>().unwrap());
                state.stack.push(1);
                Ok(())
            },
            "number_s_" =>  |tree: &mut Tree<String>,cur_node:NodeId, state: &mut Self::Tstate| {
                let e= state.stack.pop().unwrap();
                let digit = state.stack.pop().unwrap();
                let _= state.stack.pop().unwrap();
                let prev_digit = state.stack.pop().unwrap();
                let base :usize= 10;
                let res = prev_digit * base.pow(e as u32)  + digit;
                state.stack.push(res);
                state.stack.push(e+1);
                Ok(())
            },
            "print" =>  |tree: &mut Tree<String>,cur_node:NodeId, state: &mut Self::Tstate| {
                let digit = state.stack.pop().unwrap();
                println!("stack last item:{digit}");
                Ok(())
            },
            _ =>  |_tree: &mut Tree<String>,cur_node:NodeId, _state: &mut Self::Tstate| {
                Ok(())
            }
        };
        Ok(Box::new(instruction))
    }

    fn create_new_state() -> Self::Tstate {
        SimpleStackVmState { stack: vec![] }
    }
}