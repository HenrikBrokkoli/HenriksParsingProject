extern crate core;

use henriks_parsing_project::script_parser::Parser;
use henriks_parsing_project::vms::{NullVm, VM};
use std::{env, fs};

/// Main entry point for the command-line tool.
///
/// Usage: henriks-parsing-project <rule_file> <script_file>
///
/// This program takes two arguments:
/// 1. Path to a file containing grammar rules
/// 2. Path to a file containing a script to parse
///
/// It parses the script according to the grammar rules and returns the parse tree.

fn main() {
    let args: Vec<String> = env::args().collect();
    let rule_path = &args[1];
    let script_path = &args[2];
    let rules = fs::read_to_string(rule_path).expect("Unable to read rule file");
    let script = fs::read_to_string(script_path).expect("Unable to read script file");

    let vm = NullVm::new();
    let mut state = NullVm::create_new_state();
    let mut parser = Parser::new_from_text(&rules, &vm);

    let graph = parser.parse(&script, &mut state).unwrap();
}
