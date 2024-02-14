extern crate HenriksParsingProject;

use std::fs;
use HenriksParsingProject::script_parser::Parser;
use HenriksParsingProject::vms::simple_stack_vm::SimpleStackVm;
use HenriksParsingProject::vms::VM;

fn main() {

    //First read the rules that define the syntax of our language
    let rules = fs::read_to_string("examples/simple_stack_based_math.txt").expect("Unable to read rule file");
    //We use a simple Stackmachine that has a few operations, so our code language can do something.
    let vm=SimpleStackVm{};
    //The state of the stack machine is not handled by the stack machine. We have to create it.
    let mut state= SimpleStackVm::create_new_state();
    //Using the stackmachine and the rulesstring we create a parser.
    let mut parser = Parser::new(&rules,&vm);
    //In this example our script is very short and saved in a string. We want to do the following:
    //Put 1 on the stack, put two on the stack, take top two elements from stack (1 and 2) and add them and put the result on the stack
    //Then add three and then substract for.
    //Then print out the top element on the stack
    let script="1 2 + 3 + 4 - print";
    let graph = parser.parse(&script,&mut state).unwrap();
    //You should the see printed out result

    println!("ok")
}