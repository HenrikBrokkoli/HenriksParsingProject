use crate::errors::ParserError;
use crate::peekables::{ParseProcess, TPeekable};
use crate::tree::{NodeId, Tree};
use crate::vms::VM;

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

    fn execute_instruction(&self, _:&mut Tree<String>, _: NodeId,_: &Self::Tinstrution, state: &mut usize) {
        CountingVm::increment_counter(state).unwrap()
    }

    fn create_new_state() -> Self::Tstate {
        0
    }
}