use errors::ParserError;
use parse_funcs::{parse_var_name, parse_whitespace};
use peekables::{ParseProcess, TPeekable};
use vms::{Instruction, VM};

pub struct StackVmState {
    pub stack: Vec<usize>
}
pub struct StackVm {}

impl VM for StackVm {
    type Tstate = StackVmState;
    
    fn make_instruction<T>(&self, prod_name: &str, to_parse: &mut ParseProcess<T>)
                           -> Result<Box<Instruction<Self::Tstate>>, ParserError> where T: TPeekable<Item=char> {
        parse_whitespace(to_parse);
        if let Some(x)=to_parse.peek(){
            let res=parse_var_name(to_parse)?;
        }
    
       
    }

    fn create_new_state() -> Self::Tstate {
        todo!()
    }
}