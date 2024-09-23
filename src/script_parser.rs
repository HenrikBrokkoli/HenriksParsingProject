use std::borrow::Borrow;
use std::collections::HashMap;
use std::rc::Rc;
use std::str::Chars;

use parser_data::{ElementIndex, ElementType, ElementVerbose, Production};
use tree::{NodeId, Tree};
use vms::{Instruction, VM};

use crate::errors::GrammarError::MissingProduction;
use crate::errors::ParserError;
use crate::errors::ParserError::{EndOfCharsError, UnexpectedCharError};
use crate::first_sets::get_first_sets;
use crate::follow_sets::get_follow_sets;
use crate::peekables::{ParseProcess, PeekableWrapper, TPeekable};
use crate::rule_parsing::RuleParser;
use crate::sets::SetMember;
use crate::steuer_map::{get_steuermaps, NTRules};

//TODO detect left recursive rules that lead to nonterminating of get_first_sets

pub struct Parser<'a, T:>
where
    T: VM,
{
    vm: &'a T,
    discard: Option<ElementIndex>,
    rules_with_steuermaps: HashMap<ElementIndex, NTRules<T::Tstate>>,
    elements: Vec<ElementVerbose>,
}


impl<'a, T> Parser<'a, T>
where
    T: VM + 'a,
{
    pub fn new(rule_text: &str, vm: &'a T) -> Parser<'a, T> {
        let mut peekable = PeekableWrapper::<PeekableWrapper<Chars>>::new(rule_text.chars().peekable());
        let mut rule_parser = RuleParser::new(&mut peekable, vm);
        let rules = rule_parser.parse_rules().unwrap();
        let discard = rules.ignore;
        let RuleParser { vm: _, parse_process: _parse_process, parser_data } = rule_parser;
        let elements = parser_data.get_elements_verbose();
        let first_dict = get_first_sets(&parser_data).unwrap();
        let follow_dict = get_follow_sets(parser_data.get_element_nt_index("start").unwrap(), &first_dict, &parser_data).unwrap();

        let rules_with_steuermaps = get_steuermaps(&first_dict, &follow_dict, parser_data).unwrap();
        Parser { vm, discard, rules_with_steuermaps, elements }
    }

    pub fn parse(&mut self, to_parse: &'a str, state: &mut T::Tstate) -> Result<Tree<String>, ParserError> {
        let mut peekable = PeekableWrapper::<Chars>::new(to_parse.chars().peekable());
        let mut to_parse = ParseProcess::<PeekableWrapper<Chars>>::new(&mut peekable, None, None);
        let start_index = self.elements.iter().position(|r| *r == ElementVerbose::new(String::from("start"), ElementType::NonTerminal)).unwrap();
        let mut tree = Tree::new();

        self.parse_production(&mut to_parse, start_index, state, &mut tree, None)?;
        Ok(tree)
    }

    fn get_fitting_production(&self, to_parse: &mut ParseProcess<PeekableWrapper<Chars<'a>>>, el_index: ElementIndex, cur: &SetMember) -> Result<&Rc<Production>, ParserError> {
        let nt_rule = self.rules_with_steuermaps.get(&el_index).ok_or(MissingProduction { index: el_index })?;
        let fitting_production: Option<&Rc<Production>> = nt_rule.steuermap.get(cur);
        match fitting_production {
            None => { Err(UnexpectedCharError { chr: *to_parse.peek().unwrap_or(&'#'), pos: to_parse.cur_pos(), expected: nt_rule.steuermap.keys().cloned().map(|x| x.into()).collect::<Vec<String>>().join(";") }) }
            Some(fp) => { Ok(fp) }
        }
    }
    fn parse_production(&self, to_parse: &mut ParseProcess<PeekableWrapper<Chars<'a>>>, el_index: ElementIndex, state: &mut T::Tstate, tree: &mut Tree<String>, current_node: Option<NodeId>)
                        -> Result<(), ParserError> {
        let cur = SetMember::from(to_parse.peek());
        let nt_rule = self.rules_with_steuermaps.get(&el_index).ok_or(MissingProduction { index: el_index })?;
        let fitting_production: &Rc<Production>= self.get_fitting_production(to_parse, el_index, &cur)?;

        let id=tree.add_node(String::from(""),current_node)?;
     
        let prod = &**fitting_production;
        match prod {
            Production::NotEmpty(prod_not_empty) => {
                for next_element_index in prod_not_empty {
                    let elemente_next = self.elements.get(*next_element_index).unwrap().clone();

                    match elemente_next.et {
                        ElementType::Terminal => {
                            let _=tree.add_node(self.parse_terminal(to_parse, elemente_next.name.as_str())?,Some(id));
                        }
                        ElementType::NonTerminal => {
                            self.parse_production(to_parse, *next_element_index, state, tree,Some(id))?;
                        }
                    }
                }
            }
            Production::Empty => {}
        };
     
        self.run_instructions(tree,id, &nt_rule.instruction, state).map_err(|x| ParserError::VmError { message: x })?;

        Ok(())
    }

    fn run_instructions(&self, tree: &mut Tree<String>,cur_node:NodeId, instruction: &Option<Box<Instruction<T::Tstate>>>, state: &mut T::Tstate) -> Result<(), String> {
        match instruction {
            None => { Ok(()) }
            Some(instr) => {
                let func: &Instruction<T::Tstate> = instr.borrow();
                func(tree,cur_node, state)
            }
        }
    }

    fn parse_terminal(&self, to_parse: &mut ParseProcess<PeekableWrapper<Chars<'a>>>, terminal: &str) -> Result<String, ParserError> {
        for chr in terminal.chars() {
            let char_to_parse: &char = to_parse.peek().ok_or(EndOfCharsError)?;
            if *char_to_parse != chr {
                return Err(UnexpectedCharError { chr: *char_to_parse, pos: to_parse.cur_pos(), expected: String::from(chr) });
            }
            to_parse.next();
        }
        Ok(String::from(terminal))
    }
}


#[cfg(test)]
mod tests {
    use script_parser::Parser;
    use vms::{NullVm, VM};
    use vms::counting_vm::CountingVm;

    use crate::errors::ParserError;

    #[test]
    fn test_script_parser() {
        let rules =
            "start      -> identifier1 identifier2;\
            identifier1 -> \"a_terminal\"| #;
            identifier2 -> \"b_terminal\"| #;
";
        let vm = NullVm::new();
        let mut state = NullVm::create_new_state();
        let mut parser = Parser::new(rules, &vm);

        let text_to_parse = "a_terminalb_terminal";
        parser.parse(text_to_parse, &mut state).unwrap();
    }

    #[test]
    fn test_script_parser2() {
        let rules =
            "start      -> identifier1 identifier2;\
            identifier1 -> \"abcde\"| #;
            identifier2 -> \"zzzzzz\"| #;
";
        let vm = NullVm::new();
        let mut state = NullVm::create_new_state();
        let mut parser = Parser::new(rules, &vm);

        let text_to_parse = "abcdeg";
        if let Err(x) = parser.parse(text_to_parse, &mut state) {
            if let ParserError::UnexpectedCharError { chr, pos, expected: _ } = x {
                assert_eq!(5, pos);
                assert_eq!('g', chr)
            } else {
                panic!("")
            }
        } else {
            panic!()
        }
    }

    #[test]
    fn test_script_parser3() {
        let rules =
            "start      -> identifier1 identifier2;\
            identifier1 -> \"a_terminal\"| #;
            identifier2 -> \"b_terminal\"| \"c_terminal\";
";
        let vm = NullVm::new();
        let mut state = NullVm::create_new_state();
        let mut parser = Parser::new(rules, &vm);

        let text_to_parse = "a_terminalc_terminal";
        parser.parse(text_to_parse, &mut state).unwrap();
    }

    #[test]
    fn test_script_parser_graph() {
        let rules =
            "start      -> identifier1 identifier2;\
            identifier1 -> \"a_terminal\"| #;
            identifier2 -> \"b_terminal\"| \"c_terminal\";
";
        let vm = NullVm::new();
        let mut state = NullVm::create_new_state();
        let mut parser = Parser::new(rules, &vm);

        let text_to_parse = "a_terminalc_terminal";
        let graph = parser.parse(text_to_parse, &mut state).unwrap();
        println!("hallo");
        println!("{}",format!("{graph:?}"));
        
        let res = graph.get_by_path_or_none(0, vec![0, 0].into_iter()).unwrap().unwrap();
        assert_eq!("a_terminal", res.data)
    }

    #[test]
    fn test_script_parser_graph2() {
        let rules =
            "start      -> identifier1 identifier2;\
            identifier1 -> \"a_terminal\"| #;
            identifier2 -> \"b_terminal\"| \"c_terminal\";
";
        let vm = NullVm::new();
        let mut state = NullVm::create_new_state();
        let mut parser = Parser::new(rules, &vm);

        let text_to_parse = "c_terminal";
        let tree = parser.parse(text_to_parse, &mut state).unwrap();
        let res = tree.get_by_path_or_none(0, vec![0, 0].into_iter()).unwrap();
        assert!(res.is_none())
    }

    #[test]
    fn test_script_parser_graph3() {
        let rules =
            "start      -> identifier1;\
            identifier1 -> \"a\" identifier2;
            identifier2 -> \"b\";
";
        let vm = NullVm::new();
        let mut state = NullVm::create_new_state();
        let mut parser = Parser::new(rules, &vm);

        let text_to_parse = "ab";
        let graph = parser.parse(text_to_parse, &mut state).unwrap();
        let res = &graph.get_by_path_or_none(0, vec![0, 1, 0].into_iter()).unwrap().unwrap().data;
        assert_eq!("b", res)
    }

    #[test]
    fn test_parse_list() {
        let rules =
            "start      -> list;\
            list -> l_element list_s ;\
            list_s -> l_element list_s| #;\
            l_element -> \"a\";\
";
        let vm = NullVm::new();
        let mut state = NullVm::create_new_state();
        let mut parser = Parser::new(rules, &vm);

        let text_to_parse = "aaa";
        let _graph = parser.parse(text_to_parse, &mut state).unwrap();
    }

    #[test]
    fn test_parse_list2() {
        let rules =
            "start      -> list;\
            list -> l_element list_s ;\
            list_s -> l_element list_s| #;\
            l_element -> \"a\";\
";
        let vm = NullVm::new();
        let mut state = NullVm::create_new_state();
        let mut parser = Parser::new(rules, &vm);

        let text_to_parse = "a";
        let _graph = parser.parse(text_to_parse, &mut state).unwrap();
    }

    #[test]
    fn test_script_parser_discarder() {
        let rules =
            "$IGNORE: whitespace; \
            start      -> \"a\" \"b\";\
            whitespace -> \" \";\
";
        let vm = NullVm::new();
        let mut state = NullVm::create_new_state();
        let mut parser = Parser::new(rules, &vm);

        let text_to_parse = " a b ";
        let _graph = parser.parse(text_to_parse, &mut state).unwrap();
    }

    #[test]
    fn test_script_parser_discarder2() {
        let rules =
            "$IGNORE: whitespace; \
            start      -> \"a\" \"b\";\
            whitespace -> \" \"| #;\
";
        let vm = NullVm::new();
        let mut state = NullVm::create_new_state();
        let mut parser = Parser::new(rules, &vm);

        let text_to_parse = "a b";
        let _graph = parser.parse(text_to_parse, &mut state).unwrap();
    }

    #[test]
    fn test_script_parser_discarder3() {
        let rules =
            "$IGNORE: whitespaces; \
            start      -> \"a\" \"b\" \"c\" ;\
            whitespaces -> whitespace whitespaces_s ;\
            whitespaces_s -> whitespace whitespaces_s| #;\
            whitespace -> \" \";\
";

        let vm = NullVm::new();
        let mut state = NullVm::create_new_state();
        let mut parser = Parser::new(rules, &vm);

        let text_to_parse = " a  b   c  ";
        let graph = parser.parse(text_to_parse, &mut state).unwrap();

        assert_eq!("a", graph.get_by_path_or_none(0, vec![0].into_iter()).unwrap().unwrap().data);
        assert_eq!("b", graph.get_by_path_or_none(0, vec![1].into_iter()).unwrap().unwrap().data);
        assert_eq!("c", graph.get_by_path_or_none(0, vec![2].into_iter()).unwrap().unwrap().data);
    }

    #[test]
    fn test_script_parser_discarder4() {
        let rules =
            "$IGNORE: whitespaces; \
            start      -> a_or_b \"c\" ;\
            a_or_b -> \"a\"| \"b\";
            whitespaces -> whitespace whitespaces_s ;\
            whitespaces_s -> whitespace whitespaces_s| #;\
            whitespace -> \" \";\
";

        let vm = NullVm::new();
        let mut state = NullVm::create_new_state();
        let mut parser = Parser::new(rules, &vm);

        let text_to_parse = "a    c";
        let graph = parser.parse(text_to_parse, &mut state).unwrap();

        assert_eq!("a", graph.get_by_path_or_none(0, vec![0, 0].into_iter()).unwrap().unwrap().data);
        assert_eq!("c", graph.get_by_path_or_none(0, vec![1].into_iter()).unwrap().unwrap().data);
    }

    #[test]
    fn test_counting_vm() {
        let rules =
            "start      -> count count count ;\
            count -> \"a\" {} ;\
";

        let vm = CountingVm {};
        let mut state = CountingVm::create_new_state();
        let mut parser = Parser::new(rules, &vm);

        let text_to_parse = "aaa";
        let _graph = parser.parse(text_to_parse, &mut state).unwrap();
        assert_eq!(3, state);
    }
}











