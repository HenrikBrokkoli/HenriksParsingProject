extern crate core;
extern crate HenriksParsingProject;

use std::{env, fs};
use HenriksParsingProject::script_parser::Parser;
use HenriksParsingProject::vms::{NullVm,VM};




fn main() {
    let args: Vec<String> = env::args().collect();
    let rule_path = &args[1];
    let script_path = &args[2];
    let rules = fs::read_to_string(rule_path).expect("Unable to read rule file");
    let script = fs::read_to_string(script_path).expect("Unable to read script file");

    let mut vm=NullVm::new();
    let mut state= NullVm::create_new_state();
    let mut parser = Parser::new(&rules,&vm);

    let graph = parser.parse(&script,&mut state).unwrap();

}





