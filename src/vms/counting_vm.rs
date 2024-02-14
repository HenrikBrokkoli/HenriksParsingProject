use peekables::{ParseProcess, TPeekable};
use simple_graph::Graph;
use vms::{Instruction, VM};

pub struct CountingVm {}

impl CountingVm {
    pub fn increment_counter(state: &mut usize) -> Result<(), String> {
        *state += 1;
        Ok(())
    }
}

impl VM for CountingVm {
    type Tstate = usize;


    fn make_instruction<T>(&self,prod_name:&str, to_parse: &mut ParseProcess<T>) -> Result<Box<Instruction<Self::Tstate>>, String> where T: TPeekable<Item=char> {
        Ok(Box::new(move |graph, state| CountingVm::increment_counter(state)))
    }

    fn create_new_state() -> Self::Tstate {
        0
    }
}