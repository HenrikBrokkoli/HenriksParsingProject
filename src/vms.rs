use peekables::{ParseProcess, TPeekable};
use simple_graph::Graph;
use tree::{NodeId, Tree};

pub mod counting_vm;
pub mod simple_stack_vm;

pub type Instruction<T> = dyn Fn(&mut Tree<String>,NodeId, &mut T) -> Result<(), String>;


///The trait for the VM that will run the parsed instructions.

pub trait VM {
    ///The type of the state of the VM. Because it was hard to put State as an object owned by the VM they are now seperate.
    /// That`s why the type of the state of the VM has to be declared
    type Tstate;

    ///Takes the current production name of the parser and a ParseProcess as argument and returns the fitting instructions.
    /// The contents of the ParseProcess can be ignored, but it needs to be finished (calling next until no more elements)
    /// returns a boxed instruction for later use in the interpreter.
    fn make_instruction<'a, T>(&'a self, prod_name: &str, to_parse: &mut ParseProcess<T>) -> Result<Box<Instruction<Self::Tstate>>, String> where T: TPeekable<Item=char>;

    ///Create a new state to hold the state of the current VM. The Vm does not take care of the state.
    fn create_new_state() -> Self::Tstate;
}

#[derive(Debug)]
pub struct NullVm {}

impl NullVm {
    pub fn new() -> NullVm {
        NullVm {}
    }

    pub fn null_instruction(_tree: &mut Tree<String>) -> Result<(), String> {
        Ok(())
    }
}


///This VM does nothing. It`s just for testing, or if you need to parse text without doing anything with it.
impl VM for NullVm {
    type Tstate = usize;

    fn make_instruction<'a, T>(&'a self, prod_name: &str, to_parse: &mut ParseProcess<T>) -> Result<Box<Instruction<usize>>, String> where T: TPeekable<Item=char> {
        let mut c = to_parse.next();
        while let Some(_cc) = c {
            c = to_parse.next()
        }
        Ok(Box::new(|graph,_node_id, _state| NullVm::null_instruction(graph)))
    }

    fn create_new_state() -> Self::Tstate {
        0
    }
}

