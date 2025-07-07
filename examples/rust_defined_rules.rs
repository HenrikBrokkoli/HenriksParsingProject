extern crate HenriksParsingProject;

use HenriksParsingProject::parser_data::ParserData;
use HenriksParsingProject::script_parser::Parser;
use HenriksParsingProject::vms::VM;
use HenriksParsingProject::vms::stack_vm::{Instruction, StackVm};

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
    parser_data.add_production(start_idx, vec![terms_idx]);

    // terms -> term terms_s;
    parser_data.add_production(terms_idx, vec![term_idx, terms_s_idx]);

    // terms_s -> whitespace term terms_s | #;
    parser_data.add_production(terms_s_idx, vec![whitespace_idx, term_idx, terms_s_idx]);
    parser_data.add_production(terms_s_idx, vec![]);

    // term -> add|sub|number|print;
    parser_data.add_production(term_idx, vec![add_idx]);
    parser_data.add_production(term_idx, vec![sub_idx]);
    parser_data.add_production(term_idx, vec![number_idx]);
    parser_data.add_production(term_idx, vec![print_idx]);

    // print -> "print" {PopToReg;PrintReg;};
    parser_data.add_production(print_idx, vec![print_term_idx]);
    parser_data.add_instructions(
        print_idx,
        vec![Instruction::PopToReg, Instruction::PrintReg],
    );

    // add -> "+" {PopToReg; Add;};
    parser_data.add_production(add_idx, vec![plus_idx]);
    parser_data.add_instructions(add_idx, vec![Instruction::PopToReg, Instruction::Add]);

    // sub -> "-" {PopToReg;Sub;};

    parser_data.add_production(sub_idx, vec![minus_idx]);
    parser_data.add_instructions(sub_idx, vec![Instruction::PopToReg, Instruction::Sub]);

    // number-> digit number_s {PopDiscard;};
    parser_data.add_production(number_idx, vec![digit_idx, number_s_idx]);
    parser_data.add_instructions(number_idx, vec![Instruction::PopDiscard]);

    // number_s -> number_s_ | # {};
    parser_data.add_production(number_s_idx, vec![number_s__idx]);
    parser_data.add_production(number_s_idx, vec![]);

    // number_s_ -> digit number_s {PopToReg;PopToReg2;PopDiscard;Pow 10;RegAddConst 1;PushReg;};
    parser_data.add_production(number_s__idx, vec![digit_idx, number_s_idx]);
    parser_data.add_instructions(
        number_s__idx,
        vec![
            Instruction::PopToReg,
            Instruction::PopToReg2,
            Instruction::PopDiscard,
            Instruction::Pow(10),
            Instruction::RegAddConst(1),
            Instruction::PushReg,
        ],
    );

    // digit -> "0"|"1"|"2"|"3"|"4"|"5"|"6"|"7"|"8"|"9" {PushFromTree;PushConst 1;};
    parser_data.set_productions(
        digit_idx,
        vec![
            vec![digit0_idx],
            vec![digit1_idx],
            vec![digit2_idx],
            vec![digit3_idx],
            vec![digit4_idx],
            vec![digit5_idx],
            vec![digit6_idx],
            vec![digit7_idx],
            vec![digit8_idx],
            vec![digit9_idx],
        ],
    );
    parser_data.add_instructions(
        digit_idx, vec![Instruction::PushFromTree, Instruction::PushConst(1)],
    );

    // whitespace -> " ";
    parser_data.add_production(whitespace_idx, vec![space_idx]);

    // whitespaces -> whitespace whitespaces_s;
    parser_data.add_production(whitespaces_idx, vec![whitespace_idx, whitespaces_s_idx]);

    // whitespaces_s -> whitespace whitespaces_s| #;
    parser_data.add_production(whitespaces_s_idx, vec![whitespace_idx, whitespaces_s_idx]);
    parser_data.add_production(whitespaces_s_idx, vec![]);

    // Create a parser with the rules
    let mut parser = Parser::new_from_parser_data(parser_data,start_idx,&vm);

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
