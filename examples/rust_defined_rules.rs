extern crate HenriksParsingProject;

use HenriksParsingProject::first_sets::get_first_sets;
use HenriksParsingProject::follow_sets::get_follow_sets;
use HenriksParsingProject::parser_data::{
    ElementVerbose, NonTerminalRules, ParserData, Production,
};
use HenriksParsingProject::script_parser::Parser;
use HenriksParsingProject::steuer_map::get_steuermaps;
use HenriksParsingProject::vms::VM;
use HenriksParsingProject::vms::stack_vm::{Instruction, StackVm};
use std::rc::Rc;

fn main() {
    // Create a VM instance
    let vm = StackVm {};

    // Create a new ParserData instance
    let mut parser_data = ParserData::<StackVm>::new();

    // Define elements (terminals and non-terminals)
    // Start rule
    let start_idx = parser_data.get_or_add_non_terminal("start");

    // Terms
    let terms_idx = parser_data.get_or_add_non_terminal("terms");
    let terms_s_idx = parser_data.get_or_add_non_terminal("terms_s");

    // Term types
    let term_idx = parser_data.get_or_add_non_terminal("term");
    let add_idx = parser_data.get_or_add_non_terminal("add");
    let sub_idx = parser_data.get_or_add_non_terminal("sub");
    let number_idx = parser_data.get_or_add_non_terminal("number");
    let print_idx = parser_data.get_or_add_non_terminal("print");

    // Number related
    let number_s_idx = parser_data.get_or_add_non_terminal("number_s");
    let number_s__idx = parser_data.get_or_add_non_terminal("number_s_");
    let digit_idx = parser_data.get_or_add_non_terminal("digit");

    // Whitespace
    let whitespace_idx = parser_data.get_or_add_non_terminal("whitespace");
    let whitespaces_idx = parser_data.get_or_add_non_terminal("whitespaces");
    let whitespaces_s_idx = parser_data.get_or_add_non_terminal("whitespaces_s");

    // Terminals
    let plus_idx = parser_data.get_or_add_terminal("+");
    let minus_idx = parser_data.get_or_add_terminal("-");
    let print_term_idx = parser_data.get_or_add_terminal("print");
    let space_idx = parser_data.get_or_add_terminal(" ");

    // Digit terminals
    let digit0_idx = parser_data.get_or_add_terminal("0");
    let digit1_idx = parser_data.get_or_add_terminal("1");
    let digit2_idx = parser_data.get_or_add_terminal("2");
    let digit3_idx = parser_data.get_or_add_terminal("3");
    let digit4_idx = parser_data.get_or_add_terminal("4");
    let digit5_idx = parser_data.get_or_add_terminal("5");
    let digit6_idx = parser_data.get_or_add_terminal("6");
    let digit7_idx = parser_data.get_or_add_terminal("7");
    let digit8_idx = parser_data.get_or_add_terminal("8");
    let digit9_idx = parser_data.get_or_add_terminal("9");

    // Define productions

    // start -> terms;
    parser_data.add_production(start_idx, vec![term_idx]);

    // terms -> term terms_s;
    parser_data.add_production(terms_idx, vec![term_idx, terms_s_idx]);

    // terms_s -> whitespace term terms_s | #;
    parser_data.add_production(terms_s_idx, vec![whitespace_idx, term_idx, terms_s_idx]);
    parser_data.add_production(terms_s_idx, vec![]);

    // term -> add|sub|number|print;
    parser_data.add_production(term_idx, vec![add_idx]);
    parser_data.add_production(term_idx,vec![sub_idx]);
    parser_data.add_production(term_idx,vec![number_idx]);
    parser_data.add_production(term_idx,vec![print_idx]);
   

    // print -> "print" {PopToReg;PrintReg;};
    parser_data.add_production(print_idx,vec![print_term_idx]);
    parser_data.add_instructions(print_idx, vec![Instruction::PopToReg, Instruction::PrintReg],
    );
    
    

    // add -> "+" {PopToReg; Add;};
    let add_prod = Rc::new(Production::NotEmpty(vec![plus_idx]));
    let add_rules = NonTerminalRules::new(
        vec![add_prod],
        None,
        vec![Instruction::PopToReg, Instruction::Add],
    );
    parser_data.parse_rules.rules.insert(add_idx, add_rules);

    // sub -> "-" {PopToReg;Sub;};
    let sub_prod = Rc::new(Production::NotEmpty(vec![minus_idx]));
    let sub_rules = NonTerminalRules::new(
        vec![sub_prod],
        None,
        vec![Instruction::PopToReg, Instruction::Sub],
    );
    parser_data.parse_rules.rules.insert(sub_idx, sub_rules);

    // number-> digit number_s {PopDiscard;};
    let number_prod = Rc::new(Production::NotEmpty(vec![digit_idx, number_s_idx]));
    let number_rules =
        NonTerminalRules::new(vec![number_prod], None, vec![Instruction::PopDiscard]);
    parser_data
        .parse_rules
        .rules
        .insert(number_idx, number_rules);

    // number_s -> number_s_ | # {};
    let number_s_prod1 = Rc::new(Production::NotEmpty(vec![number_s__idx]));
    let number_s_prod2 = Rc::new(Production::Empty);
    let number_s_rules = NonTerminalRules {
        possible_productions: vec![number_s_prod1, number_s_prod2],
        ignore: None,
        instruction: vec![],
    };
    parser_data
        .parse_rules
        .rules
        .insert(number_s_idx, number_s_rules);

    // number_s_ -> digit number_s {PopToReg;PopToReg2;PopDiscard;Pow 10;RegAddConst 1;PushReg;};
    let number_s__prod = Rc::new(Production::NotEmpty(vec![digit_idx, number_s_idx]));
    let number_s__rules = NonTerminalRules {
        possible_productions: vec![number_s__prod],
        ignore: None,
        instruction: vec![
            Instruction::PopToReg,
            Instruction::PopToReg2,
            Instruction::PopDiscard,
            Instruction::Pow(10),
            Instruction::RegAddConst(1),
            Instruction::PushReg,
        ],
    };
    parser_data
        .parse_rules
        .rules
        .insert(number_s__idx, number_s__rules);

    // digit -> "0"|"1"|"2"|"3"|"4"|"5"|"6"|"7"|"8"|"9" {PushFromTree;PushConst 1;};
    let digit_prod0 = Rc::new(Production::NotEmpty(vec![digit0_idx]));
    let digit_prod1 = Rc::new(Production::NotEmpty(vec![digit1_idx]));
    let digit_prod2 = Rc::new(Production::NotEmpty(vec![digit2_idx]));
    let digit_prod3 = Rc::new(Production::NotEmpty(vec![digit3_idx]));
    let digit_prod4 = Rc::new(Production::NotEmpty(vec![digit4_idx]));
    let digit_prod5 = Rc::new(Production::NotEmpty(vec![digit5_idx]));
    let digit_prod6 = Rc::new(Production::NotEmpty(vec![digit6_idx]));
    let digit_prod7 = Rc::new(Production::NotEmpty(vec![digit7_idx]));
    let digit_prod8 = Rc::new(Production::NotEmpty(vec![digit8_idx]));
    let digit_prod9 = Rc::new(Production::NotEmpty(vec![digit9_idx]));
    let digit_rules = NonTerminalRules {
        possible_productions: vec![
            digit_prod0,
            digit_prod1,
            digit_prod2,
            digit_prod3,
            digit_prod4,
            digit_prod5,
            digit_prod6,
            digit_prod7,
            digit_prod8,
            digit_prod9,
        ],
        ignore: None,
        instruction: vec![Instruction::PushFromTree, Instruction::PushConst(1)],
    };
    parser_data.parse_rules.rules.insert(digit_idx, digit_rules);

    // whitespace -> " ";
    let whitespace_prod = Rc::new(Production::NotEmpty(vec![space_idx]));
    let whitespace_rules = NonTerminalRules {
        possible_productions: vec![whitespace_prod],
        ignore: None,
        instruction: vec![],
    };
    parser_data
        .parse_rules
        .rules
        .insert(whitespace_idx, whitespace_rules);

    // whitespaces -> whitespace whitespaces_s;
    let whitespaces_prod = Rc::new(Production::NotEmpty(vec![
        whitespace_idx,
        whitespaces_s_idx,
    ]));
    let whitespaces_rules = NonTerminalRules {
        possible_productions: vec![whitespaces_prod],
        ignore: None,
        instruction: vec![],
    };
    parser_data
        .parse_rules
        .rules
        .insert(whitespaces_idx, whitespaces_rules);

    // whitespaces_s -> whitespace whitespaces_s| #;
    let whitespaces_s_prod1 = Rc::new(Production::NotEmpty(vec![
        whitespace_idx,
        whitespaces_s_idx,
    ]));
    let whitespaces_s_prod2 = Rc::new(Production::Empty);
    let whitespaces_s_rules = NonTerminalRules {
        possible_productions: vec![whitespaces_s_prod1, whitespaces_s_prod2],
        ignore: None,
        instruction: vec![],
    };
    parser_data
        .parse_rules
        .rules
        .insert(whitespaces_s_idx, whitespaces_s_rules);

    // Calculate first and follow sets
    let first_dict = get_first_sets(&parser_data).unwrap();
    let follow_dict = get_follow_sets(start_idx, &first_dict, &parser_data).unwrap();

    let elements_verbose = parser_data.get_elements_verbose();
    // Get steuer maps
    let rules_with_steuermaps = get_steuermaps(&first_dict, &follow_dict, parser_data).unwrap();

    // Create a parser with the rules
    let mut parser = Parser::new(rules_with_steuermaps, elements_verbose, &vm);

    // Create a VM state
    let mut state = StackVm::create_new_state();

    // Parse a script
    let script = "1 2 + 3 + 4 - print";
    let result = parser.parse(&script, &mut state);

    match result {
        Ok(_) => println!("Parsing successful!"),
        Err(e) => println!("Parsing error: {:?}", e),
    }

    println!("ok");
}
