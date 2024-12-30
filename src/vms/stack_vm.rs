use errors::ParserError;
use parse_funcs::{parse_var_name, parse_whitespace};
use peekables::{ParseProcess, TPeekable};
use tree::{NodeId, Tree};
use vms::VM;

pub struct StackVmState {
    pub stack: Vec<usize>
}
pub struct StackVm {}

impl VM for StackVm {
    type Tstate = StackVmState;
    type Tinstrution = ();

    fn parse_instructions<'a, T>(&'a self, prod_name: &str, to_parse: &mut ParseProcess<T>) -> Result<Vec<Self::Tinstrution>, ParserError>
    where
        T: TPeekable<Item=char>
    {
        parse_whitespace(to_parse);
        if let Some(x)=to_parse.peek(){
            let res=parse_var_name(to_parse)?;
        }
        todo!()
    }


    fn execute_instruction(&self, tree: &mut Tree<String>, cur_node: NodeId, instruction: &Self::Tinstrution, state: &mut Self::Tstate) {
        todo!()
    }

    fn create_new_state() -> Self::Tstate {
        todo!()
    }
}