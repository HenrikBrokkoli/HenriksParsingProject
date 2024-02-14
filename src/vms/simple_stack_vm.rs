use peekables::{ParseProcess, TPeekable};
use simple_graph::Graph;
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

        let instruction = match prod_name {
            "add" => move |graph: &mut Graph<String>, state: &mut Self::Tstate| {
                let res = state.stack.pop().unwrap() + state.stack.pop().unwrap();
                state.stack.push(res);

                Ok(())
            },
            "sub" => move |graph: &mut Graph<String>, state: &mut Self::Tstate| {
                let second = state.stack.pop().unwrap();
                let res = state.stack.pop().unwrap() - second;
                state.stack.push(res);
                Ok(())
            },
            "number" => move |graph: &mut Graph<String>, state: &mut Self::Tstate| {
                Ok(())
            },
            "digit" => move |graph: &mut Graph<String>, state: &mut Self::Tstate| {
                state.stack.push(graph.find_node(0, 0).unwrap().data.parse::<usize>().unwrap());
                Ok(())
            },
            "number_s_" => move |graph: &mut Graph<String>, state: &mut Self::Tstate| {
                let digit = state.stack.pop().unwrap();
                let prev_digit = state.stack.pop().unwrap();
                let res = prev_digit * 10 + digit;
                state.stack.push(res);
                Ok(())
            },
            "print" => move |graph: &mut Graph<String>, state: &mut Self::Tstate| {
                let digit = state.stack.pop().unwrap();
                println!("stack last item:{digit}");
                Ok(())
            },
            _ => move |_graph: &mut Graph<String>, _state: &mut Self::Tstate| {
                Ok(())
            }
        };
        Ok(Box::new(instruction))
    }

    fn create_new_state() -> Self::Tstate {
        SimpleStackVmState { stack: vec![] }
    }
}