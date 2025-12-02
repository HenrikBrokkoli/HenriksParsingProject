extern crate henriks_parsing_project;

use henriks_parsing_project::script_parser::Parser;
use henriks_parsing_project::vms::VM;
use henriks_parsing_project::vms::stack_vm::StackVm;
use std::fs;

fn main() {
    //First read the rules that define the syntax of our language
    let rules =
        fs::read_to_string("examples/stack_based_math.txt").expect("Unable to read rule file");
    //We use a simple Stackmachine that has a few operations, so our code language can do something.
    let vm = StackVm {};
    //The state of the stack machine is not handled by the stack machine. We have to create it.
    let mut state = StackVm::create_new_state();
    //Using the stackmachine and the rulestring we create a parser.
    let mut parser = Parser::new_from_text(&rules, &vm);
    //In this example our script is very short and saved in a string. We want to do the following:
    //Put 1 on the stack, put two on the stack, take top two elements from stack (1 and 2) and add them and put the result on the stack
    //Then add three and then subtract four.
    //Then print out the top element on the stack
    let script = "1 2 + 3 + 4 - print";
    let _ = parser.parse(&script, &mut state).unwrap();
    //You should the see printed out result

    println!("ok")
}
