use errors::ParserError;
use peekables::{ParseProcess, TPeekable};
use tree::{NodeId, Tree};
use vms::VM;

pub struct CountingVm {}

impl CountingVm {
    pub fn increment_counter(state: &mut usize) -> Result<(), String> {
        *state += 1;
        Ok(())
    }
}

impl VM for CountingVm {
    type Tstate = usize;
    type Tinstrution = usize;


    fn parse_instructions<T>(&self, _prod_name:&str, _to_parse: &mut ParseProcess<T>) -> Result<Vec<Self::Tinstrution>, ParserError> where T: TPeekable<Item=char> {
        Ok(vec![0])
    }

    fn execute_instruction(&self, tree:&mut Tree<String>, cur_node: NodeId,instruction: &Self::Tinstrution, state: &mut usize) {
        CountingVm::increment_counter(state).unwrap()
    }

    fn create_new_state() -> Self::Tstate {
        0
    }
}