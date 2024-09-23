use std::borrow::Borrow;
use std::collections::{HashMap, HashSet};
use std::mem;
use std::rc::Rc;
use std::str::Chars;

use parser_data::{ElementIndex, ElementType, ElementVerbose, NonTerminalRules, ParserData, ParseRules, PossibleProductions, Ppp, Production, RuleMap};
use vms::{Instruction, VM};

use crate::errors::GrammarError::UnexpectedElementError;
use crate::errors::ParserError;
use crate::errors::ParserError::{EndOfCharsError, UnexpectedCharError};
use crate::parse_funcs::{parse_symbol, parse_var_name, parse_whitespace};
use crate::peekables::{ParseProcess, PeekableWrapper, TPeekable};

//use crate::virtual_machine::{VirtualMachine};


pub struct RuleParser<'vm, 'pp, T> where T: VM + 'vm {
    pub vm: &'vm T,
    pub parse_process: Ppp<'pp>,
    pub parser_data: ParserData<T>,
    //pub parse_rules: ParseRules<T>,
    //elments: Vec<ET>,
    //element_data: HashMap<ET, ElementData>,
}

impl<'vm, 'pp, T> RuleParser<'vm, 'pp, T> where T: VM + 'vm {
    pub fn new(peekable: &'pp mut PeekableWrapper<Chars<'pp>>, vm: &'vm T) -> RuleParser<'vm, 'pp, T> {
        let parse_process = ParseProcess::<PeekableWrapper<Chars>>::new(peekable, None, None);
        let parser_data = ParserData::<T>::new();
        RuleParser {
            vm,
            parse_process,
            parser_data,
        }
    }

    pub fn parse_rules(&mut self) -> Result<&ParseRules<T>, ParserError> {
        self.parse_special()?;
        loop {
            parse_whitespace(&mut self.parse_process);
            if self.parse_process.peek().is_none() {
                break;
            }
            let (rule_name, rule) = self.parse_rule()?;
            self.merge_rule(rule, rule_name);
        }
        self.edit_rules()?;
        Ok(&self.parser_data.parse_rules)
    }

    fn parse_special(&mut self) -> Result<(), ParserError> {
        if parse_symbol(&mut self.parse_process, '$').is_ok() {
            let special_instruction = parse_var_name(&mut self.parse_process)?;
            parse_symbol(&mut self.parse_process, ':')?;
            parse_whitespace(&mut self.parse_process);
            if special_instruction == "IGNORE" {
                if parse_symbol(&mut self.parse_process, '#').is_ok() {
                    self.parser_data.parse_rules.ignore = None
                } else {
                    let ignore_name = parse_var_name(&mut self.parse_process)?;
                    let key = self.parser_data.get_or_add_element_key(&ElementVerbose::new(ignore_name,ElementType::NonTerminal));
                    self.parser_data.parse_rules.ignore = Some(key);
                }
            } else { return Err(ParserError::UnknownSpecialOperation { operation: special_instruction, pos: self.parse_process.cur_pos() }); }
            parse_symbol(&mut self.parse_process, ';')?;
        };

        Ok(())
    }
    fn edit_rules(&mut self) -> Result<(), ParserError> {
        let mut edited_rules = RuleMap::new();
        struct Action {
            weave_in: bool,
        }

        //First we iterate over the rules and see what actions to do. Then we take all rules iterate a second time to do the actions on them

        //collect actions to do on the rules in a hashmap
        let mut actions = HashMap::new();
        for (rule_index, rule) in self.parser_data.parse_rules.rules.iter() {
            let mut weave_in: bool = false;
            if let Some(ignore) = &rule.ignore {
                if !check_is_derivative(&self.parser_data.element_types, &self.parser_data.parse_rules, *ignore, *rule_index)? {
                    weave_in = true;
                }
            }
            let action = Action { weave_in };
            actions.insert(rule_index.clone(), action);
        }

        //take rules iterate over them and edit them.
        let rules_to_edit = mem::take(&mut self.parser_data.parse_rules.rules);
        for (rule_name, rule) in rules_to_edit.into_iter() {
            let action = actions.get(&rule_name).ok_or(ParserError::InternalError { message: format!("keine Action vorhandne für {rule_name}") })?;//should never happen because in the first iteration we made an entry for every rule
            let possible_productions: PossibleProductions;
            let mut ignore_new = rule.ignore.clone();
            if let Some(ignore) = &rule.ignore {
                if action.weave_in {
                    possible_productions = self.weave_in_ignorers(&rule, *ignore);
                } else {
                    ignore_new = None;
                    possible_productions = rule.possible_productions;
                }
            } else {
                possible_productions = rule.possible_productions;
            }
            let new_rul = NonTerminalRules::<T> { possible_productions, ignore: ignore_new, instruction: rule.instruction };

            edited_rules.insert(rule_name.clone(), new_rul);
        }

        //put rules back
        self.parser_data.parse_rules.rules = edited_rules;
        Ok(())
    }


    fn merge_rule(&mut self, mut rule: NonTerminalRules<T>, rule_key: ElementIndex) {
        if let Some(rule_to_change) = self.parser_data.parse_rules.rules.get_mut(&rule_key) {
            rule_to_change.possible_productions.append(&mut rule.possible_productions)
        } else {
            self.parser_data.parse_rules.rules.insert(rule_key, rule);
        }
    }
    
    pub fn parse_rule(&mut self) -> Result<(ElementIndex, NonTerminalRules<T>), ParserError> {
        let identifier = parse_var_name(&mut self.parse_process)?;

        let key = self.parser_data.get_or_add_element_key(&ElementVerbose::new(identifier.clone(),ElementType::NonTerminal));

        parse_whitespace(&mut self.parse_process);
        parse_symbol(&mut self.parse_process, '-')?;
        parse_symbol(&mut self.parse_process, '>')?;
        parse_whitespace(&mut self.parse_process);
        let ignore_this_maybe = self.parse_overrides()?;
        let productions: PossibleProductions = self.parse_possible_productions()?;
        let instruction = self.parse_instruction_section(&identifier)?;
        parse_whitespace(&mut self.parse_process);
        parse_symbol(&mut self.parse_process, ';')?;
        let nt_rules = NonTerminalRules::<T> { possible_productions: productions, ignore: ignore_this_maybe, instruction };
        Ok((key, nt_rules))
    }


    fn weave_in_ignorers(&self, rule: &NonTerminalRules<T>, ignore: ElementIndex) -> Vec<Rc<Production>> {
        let mut weaved_productions = vec![];
        for production in rule.possible_productions.iter() {
            let weaved_production = Rc::new(self.weave_in_ignorers_single_production(production.clone(), ignore));
            weaved_productions.push(weaved_production);
        }


        weaved_productions
    }

    fn weave_in_ignorers_single_production(&self, production: Rc<Production>, to_weave_in: ElementIndex) -> Production {
        match production.borrow() {
            Production::NotEmpty(prod) => {
                let mut weaved_production = vec![];
                //weaved_production.push(to_weave_in.clone()); cant insert here or two weaved ignorers will be next to each other and that will lead to bugs
                for el in prod {
                    weaved_production.push(el.clone());
                    weaved_production.push(to_weave_in);
                }
                weaved_production.pop();


                Production::NotEmpty(weaved_production)
            }
            Production::Empty => { Production::Empty }//cant add weave in here or bugs
        }
    }

    fn parse_overrides(&mut self) -> Result<Option<ElementIndex>, ParserError> {
        let mut ignore_this = self.parser_data.parse_rules.ignore.clone();
        if parse_symbol(&mut self.parse_process, '$').is_ok() {
            parse_symbol(&mut self.parse_process, '[')?;
            let varname = parse_var_name(&mut self.parse_process)?;
            parse_whitespace(&mut self.parse_process);
            parse_symbol(&mut self.parse_process, ':')?;


            if varname == "IGNORE" {
                if parse_symbol(&mut self.parse_process, '#').is_ok() {
                    ignore_this = None;
                } else {
                    ignore_this = Some(self.parser_data.get_or_add_element_key(&ElementVerbose::new(parse_var_name(&mut self.parse_process)?,ElementType::NonTerminal)));
                }
            }

            parse_symbol(&mut self.parse_process, ']')?;
        };

        Ok(ignore_this)
    }

    fn parse_instruction_section(&mut self, prod_name: &str) -> Result<Option<Box<Instruction<T::Tstate>>>, ParserError> {
        match parse_symbol(&mut self.parse_process, '{') {
            Ok(_) => {}
            Err(_) => { return Ok(None); }
        };
        parse_whitespace(&mut self.parse_process);

        let mut g = ParseProcess::new(&mut self.parse_process, Some('}'), Some('\\'));
        let instruction = self.vm.make_instruction(prod_name, &mut g);
        parse_symbol(&mut self.parse_process, '}')?;
        match instruction {
            Ok(instr) => Ok(Some(instr)),
            Err(msg) => Err(ParserError::VmError { message: msg })
        }
    }

    pub fn parse_possible_productions(&mut self) -> Result<PossibleProductions, ParserError> {
        let mut elements = vec![self.parse_production()?];
        loop {
            parse_whitespace(&mut self.parse_process);
            let _ = parse_symbol(&mut self.parse_process, '\n');
            parse_whitespace(&mut self.parse_process);
            if parse_symbol(&mut self.parse_process, '|').is_ok() {
                parse_whitespace(&mut self.parse_process);
                elements.push(self.parse_production()?);
            } else {
                break;
            }
        }
        Ok(elements)
    }

    pub fn parse_production(&mut self) -> Result<Rc<Production>, ParserError> {
        let mut result = vec![];
        loop {
            parse_whitespace(&mut self.parse_process);
            if let Ok(element) = RuleParser::<'vm, 'pp, T>::parse_element(&mut self.parse_process) {
                match element {
                    None => {
                        if !result.is_empty() {
                            return Err(ParserError::GramError { err: UnexpectedElementError { reason: String::from("Leereelemente für leere Menge darf nicht mit anderen Elementen zusammen stehen"), pos: self.parse_process.cur_pos() } });
                        }
                        return Ok(Rc::new(Production::Empty));
                    }
                    Some(el) => {
                        let index = self.parser_data.get_or_add_element_key(&el);
                        result.push(index);
                    }
                }
            } else {
                break;
            }
        }

        if !result.is_empty() {
            return Ok(Rc::new(Production::NotEmpty(result)));
        }
        Err(ParserError::GramError { err: UnexpectedElementError { reason: String::from("Es wurden keine Elemente gefunden. Es müssen aber welche gefunden werden oder es muss das Symbol für leere Menge genutzt werden (#)"), pos: self.parse_process.cur_pos() } })
    }

    pub fn parse_element(to_parse: &mut ParseProcess<PeekableWrapper<Chars>>) -> Result<Option<ElementVerbose>, ParserError> {
        match to_parse.peek() {
            Some('#') => {
                to_parse.next();
                Ok(None)
            }
            Some('"') => Ok(Some(ElementVerbose::new(parse_terminal(to_parse)?,ElementType::Terminal))),
            Some(x) if x.is_alphabetic() => Ok(Some(ElementVerbose::new(parse_var_name(to_parse)?,ElementType::NonTerminal))),
            Some(x) => Err(UnexpectedCharError { chr: *x, pos: to_parse.cur_pos(), expected: String::from("char # for empty, \" for terminal ort alphabetic for element") }),
            _ => Err(EndOfCharsError)
        }
    }
}

pub fn parse_terminal(to_parse: &mut ParseProcess<PeekableWrapper<Chars>>) -> Result<String, ParserError> {
    parse_symbol(to_parse, '"')?;
    let mut literal = "".to_owned();
    let mut escape = false;
    let mut cur_char = to_parse.peek().ok_or(EndOfCharsError)?;
    while *cur_char != '"' || escape {
        if *cur_char == '\\' && !escape {
            escape = true
        } else {
            escape = false
        }
        literal.push(to_parse.next().ok_or(EndOfCharsError)?);
        cur_char = to_parse.peek().ok_or(EndOfCharsError)?
    }
    to_parse.next();//discard trailing quote
    Ok(literal)
}

fn check_is_derivative<T>(element_types: &Vec<ElementType>, parse_rules: &ParseRules<T>, left: ElementIndex, right: ElementIndex) -> Result<bool, ParserError> where T: VM {
    _check_is_derivative(element_types,parse_rules, left, right, &mut HashSet::new())
}

fn _check_is_derivative<T>(element_types: &Vec<ElementType>, parse_rules: &ParseRules<T>, left: ElementIndex, right: ElementIndex, visited: &mut HashSet<ElementIndex>) -> Result<bool, ParserError> where T: VM {
    if let ElementType::Terminal=element_types.get(left).unwrap() {
        return Ok(false)
    }
    let prods = &parse_rules.rules.get(&left).ok_or(ParserError::InternalError { message: format!("could not find production for index {left}") })?.possible_productions;

    if left == right {
        return Ok(true);
    }
    if visited.contains(&left) {
        return Ok(false);
    }
    visited.insert(left.clone());

    let mut is_der = false;
    for prod in prods {
        match &**prod {
            Production::NotEmpty(prod_ne, ..) => {
                for el in prod_ne {
                    if *el == right {
                        return Ok(true);
                    } else {
                        is_der = is_der || _check_is_derivative(element_types,parse_rules, *el, right, visited)?;
                    }
                }
            }
            Production::Empty => {}
        }
    }
    Ok(is_der)
}


#[cfg(test)]
mod tests {
    use vms::NullVm;

    use super::*;

    #[test]
    fn test_parse_terminal() {
        let mut peekable = PeekableWrapper::<Chars>::new("\"contents\"".chars().peekable());
        let mut to_parse = ParseProcess::<PeekableWrapper<Chars>>::new(&mut peekable, None, None);
        let result = parse_terminal(&mut to_parse);
        assert_eq!("contents", result.unwrap());
        assert_eq!("", to_parse.collect::<String>())
    }

    #[test]
    fn test_parse_terminal_escaped_quote() {
        let mut peekable = PeekableWrapper::<Chars>::new("\"cont\\\"ents\"".chars().peekable());
        let mut to_parse = ParseProcess::<PeekableWrapper<Chars>>::new(&mut peekable, None, None);
        let result = parse_terminal(&mut to_parse);
        assert_eq!("cont\\\"ents", result.unwrap());
        assert_eq!("", to_parse.collect::<String>())
    }

    #[test]
    fn test_parse_terminal_escaped_quote2() {
        let mut peekable = PeekableWrapper::<Chars>::new("\"cont\"ents\"".chars().peekable());
        let mut to_parse = ParseProcess::<PeekableWrapper<Chars>>::new(&mut peekable, None, None);
        let result = parse_terminal(&mut to_parse);
        assert_eq!("cont", result.unwrap());
        assert_eq!("ents\"", to_parse.collect::<String>())
    }

    #[test]
    fn test_parse_terminal_escaped_quote_and_new_line() {
        let mut peekable = PeekableWrapper::<Chars>::new("\"asdf\\n\\\"sdf\"".chars().peekable());
        let mut to_parse = ParseProcess::<PeekableWrapper<Chars>>::new(&mut peekable, None, None);
        let result = parse_terminal(&mut to_parse);
        assert_eq!("asdf\\n\\\"sdf", result.unwrap());
        assert_eq!("", to_parse.collect::<String>())
    }

    #[test]
    #[should_panic]
    fn test_parse_terminal_fail_on_missing_trailing_quote() {
        let mut peekable = PeekableWrapper::<Chars>::new("\"content".chars().peekable());
        let mut to_parse = ParseProcess::<PeekableWrapper<Chars>>::new(&mut peekable, None, None);
        let result = parse_terminal(&mut to_parse);
        result.unwrap();
    }


    #[test]
    fn test_parse_identifier() {
        let mut peekable = PeekableWrapper::<Chars>::new("IDENTIFIER".chars().peekable());
        let mut to_parse = ParseProcess::<PeekableWrapper<Chars>>::new(&mut peekable, None, None);
        let result = parse_var_name(&mut to_parse);
        assert_eq!("IDENTIFIER", result.unwrap());
        assert_eq!("", to_parse.collect::<String>())
    }

    #[test]
    fn test_parse_identifier2() {
        let mut peekable = PeekableWrapper::<Chars>::new("identifier".chars().peekable());
        let mut to_parse = ParseProcess::<PeekableWrapper<Chars>>::new(&mut peekable, None, None);
        let result = parse_var_name(&mut to_parse);
        assert_eq!("identifier", result.unwrap());
        assert_eq!("", to_parse.collect::<String>())
    }

    #[test]
    fn test_parse_identifier3() {
        let mut peekable = PeekableWrapper::<Chars>::new("identifier not".chars().peekable());
        let mut to_parse = ParseProcess::<PeekableWrapper<Chars>>::new(&mut peekable, None, None);
        let result = parse_var_name(&mut to_parse);
        assert_eq!("identifier", result.unwrap());
        assert_eq!(" not", to_parse.collect::<String>())
    }

    #[test]
    fn test_parse_production() {
        let mut peekable = PeekableWrapper::<Chars>::new("identifier \"terminal\" identifier2 \"terminal2\"".chars().peekable());

        let vm = NullVm::new();
        let mut parser = RuleParser::new(&mut peekable, &vm);
        let result = &*parser.parse_production().unwrap();
        let result = match result {
            Production::Empty => { panic!() }
            Production::NotEmpty(x, ..) => x
        };
        let get_element = |x: usize| parser.parser_data.get_element_verbose(*result.get(x).unwrap()).unwrap();

        assert_eq!(ElementVerbose::new("identifier".to_string(),ElementType::NonTerminal), get_element(0));
        assert_eq!(ElementVerbose::new("terminal".to_string(),ElementType::Terminal), get_element(1));
        assert_eq!(ElementVerbose::new("identifier2".to_string(),ElementType::NonTerminal), get_element(2));
        assert_eq!(ElementVerbose::new("terminal2".to_string(),ElementType::Terminal), get_element(3));
    }

    #[test]
    fn test_parse_production2() {
        let mut peekable = PeekableWrapper::<Chars>::new("identifier \"terminal\"
        |   identifier2 \"terminal\"".chars().peekable());

        let vm = NullVm::new();
        let mut parser = RuleParser::new(&mut peekable, &vm);
        let result = parser.parse_possible_productions().unwrap();
        let first_production = &**result.get(0).unwrap();
        let second_production = &**result.get(1).unwrap();
        let result_first = match first_production {
            Production::Empty => { panic!() }
            Production::NotEmpty(x, ..) => x
        };
        let result_second = match second_production {
            Production::Empty => { panic!() }
            Production::NotEmpty(x, ..) => x
        };

        let get_element = |x: usize, prod: &Vec<ElementIndex>| parser.parser_data.get_element_verbose(*prod.get(x).unwrap()).unwrap();

        assert_eq!(ElementVerbose::new("identifier".to_string(),ElementType::NonTerminal), get_element(0, result_first));
        assert_eq!(ElementVerbose::new("terminal".to_string(),ElementType::Terminal), get_element(1, result_first));
        assert_eq!(ElementVerbose::new("identifier2".to_string(), ElementType::NonTerminal), get_element(0, result_second));
        assert_eq!(ElementVerbose::new("terminal".to_string(),ElementType::Terminal), get_element(1, result_second));
    }

    #[test]
    fn test_parse_rule() {
        let to_parse = "identifier ->  identifier2;";
        let mut peekable = PeekableWrapper::<PeekableWrapper<Chars>>::new(to_parse.chars().peekable());
        let vm = NullVm::new();
        let mut parser = RuleParser::new(&mut peekable, &vm);
        let (rule_name, rule) = parser.parse_rule().unwrap();
        assert_eq!(ElementVerbose::new(String::from("identifier"),ElementType::NonTerminal), parser.parser_data.get_element_verbose(rule_name).unwrap());
        let first = &**rule.possible_productions.get(0).unwrap();
        let result = match first {
            Production::Empty => { panic!() }
            Production::NotEmpty(x, ..) => x
        };
        assert_eq!(ElementVerbose::new("identifier2".to_string(),ElementType::NonTerminal), parser.parser_data.get_element_verbose(*result.get(0).unwrap()).unwrap());
    }

    #[test]
    fn test_parse_rules() {
        let to_parse =
            "rule1      -> rule2
               |rule3;
rule2 -> \"b_terminal\"\
               | \"c_terminal\";";
        let mut peekable = PeekableWrapper::<PeekableWrapper<Chars>>::new(to_parse.chars().peekable());
        let vm = NullVm::new();
        let mut parser = RuleParser::new(&mut peekable, &vm);

        let _ = parser.parse_rules();
        let rules = &parser.parser_data.parse_rules.rules;
        let rule1 = rules.get(&0).unwrap();
        let rule2 = rules.get(&1).unwrap();
        let prod = &**rule1.possible_productions.get(0).unwrap();
        let rule1_production1 = match prod {
            Production::Empty => { panic!() }
            Production::NotEmpty(x, ..) => x
        };
        let prod = &**rule1.possible_productions.get(1).unwrap();
        let rule1_production2 = match prod {
            Production::Empty => { panic!() }
            Production::NotEmpty(x, ..) => x
        };
        let prod = &**rule2.possible_productions.get(0).unwrap();
        let rule2_production1 = match prod {
            Production::Empty => { panic!() }
            Production::NotEmpty(x, ..) => x
        };
        let prod = &**rule2.possible_productions.get(1).unwrap();
        let rule2_production2 = match prod {
            Production::Empty => { panic!() }
            Production::NotEmpty(x, ..) => x
        };

        let get_element = |x: usize, prod: &Vec<ElementIndex>| parser.parser_data.get_element_verbose(*prod.get(x).unwrap()).unwrap();

        assert_eq!(ElementVerbose::new("rule2".to_string(),ElementType::NonTerminal), get_element(0, rule1_production1));
        assert_eq!(ElementVerbose::new("rule3".to_string(),ElementType::NonTerminal), get_element(0, rule1_production2));
        assert_eq!(ElementVerbose::new("b_terminal".to_string(),ElementType::Terminal), get_element(0, rule2_production1));
        assert_eq!(ElementVerbose::new("c_terminal".to_string(),ElementType::Terminal), get_element(0, rule2_production2));
    }

    #[test]
    #[should_panic]
    fn test_parse_rules2() {
        let to_parse =
            "rule1      -> rule2
               |rule3;
rule2 -> \"b_terminal\"\
               | \"c_terminal\"; asdf";

        let mut peekable = PeekableWrapper::<PeekableWrapper<Chars>>::new(to_parse.chars().peekable());
        let vm = NullVm::new();
        let mut parser = RuleParser::new(&mut peekable, &vm);
        let _rules = &parser.parse_rules().unwrap().rules;
    }


    #[test]
    fn test_parse_rules3() {
        let to_parse =
            "start ->   identifier2
                        |identifier3
                        |\"a_terminal\";\
            identifier2 -> \"b_terminal\"\
                            | #;
            identifier3 -> \"c_terminal\";
";
        let mut peekable = PeekableWrapper::<PeekableWrapper<Chars>>::new(to_parse.chars().peekable());
        let vm = NullVm::new();
        let mut parser = RuleParser::new(&mut peekable, &vm);
        let _rules = &parser.parse_rules().unwrap().rules;
    }


    fn get_elements_or_panic<'a, T>(parser: &'a RuleParser<NullVm>, rules: &NonTerminalRules<T>, index: ElementIndex) -> Vec<ElementVerbose> where T: VM {
        let production = &**rules.possible_productions.get(index).unwrap();
        match production {
            Production::Empty => { panic!() }
            Production::NotEmpty(x, ..) => x.iter().map(|ei| parser.parser_data.get_element_verbose(*ei).unwrap())
        }.collect::<Vec<ElementVerbose>>()
    }

    #[test]
    fn test_parse_rules4() {
        let to_parse =
            "rule1      -> rule2
               |rule3;

rule2 -> \"b_terminal\"\
               | \"c_terminal\";\
               ";

        let mut peekable = PeekableWrapper::<PeekableWrapper<Chars>>::new(to_parse.chars().peekable());
        let vm = NullVm::new();
        let mut parser = RuleParser::new(&mut peekable, &vm);
        let _ = parser.parse_rules();
        let rules = &parser.parser_data.parse_rules.rules;
        let rule1 = rules.get(&0).unwrap();
        let rule2 = rules.get(&1).unwrap();
        let rule1_production1 = get_elements_or_panic(&parser, rule1, 0);
        let rule1_production2 = get_elements_or_panic(&parser, rule1, 1);
        let rule2_production1 = get_elements_or_panic(&parser, rule2, 0);
        let rule2_production2 = get_elements_or_panic(&parser, rule2, 1);
        assert_eq!(ElementVerbose::new("rule2".to_string(),ElementType::NonTerminal), rule1_production1[0]);
        assert_eq!(ElementVerbose::new("rule3".to_string(),ElementType::NonTerminal), rule1_production2[0]);
        assert_eq!(ElementVerbose::new("b_terminal".to_string(),ElementType::Terminal), rule2_production1[0]);
        assert_eq!(ElementVerbose::new("c_terminal".to_string(),ElementType::Terminal), rule2_production2[0]);
    }

    #[test]
    fn test_parse_rules5() {
        let to_parse =
            "start      -> not_end | #;\
            not_end -> \"a\" start ;\
";
        let mut peekable = PeekableWrapper::<PeekableWrapper<Chars>>::new(to_parse.chars().peekable());
        let vm = NullVm::new();
        let mut parser = RuleParser::new(&mut peekable, &vm);
        let _ = parser.parse_rules();
        let rules = &parser.parser_data.parse_rules.rules;
        let start = rules.get(&0).unwrap();
        let not_end = rules.get(&1).unwrap();
        let start_prod = get_elements_or_panic(&parser, start, 0);
        let not_end_prod = get_elements_or_panic(&parser, not_end, 0);
        assert_eq!(2, start.possible_productions.len());
        assert_eq!(1, not_end.possible_productions.len());

        assert_eq!(ElementVerbose::new("not_end".to_string(),ElementType::NonTerminal), start_prod[0]);
        assert_eq!(ElementVerbose::new("start".to_string(),ElementType::NonTerminal), not_end_prod[1]);
    }

    #[test]
    fn test_rule_parser_discarder() {
        let to_parse =
            "$IGNORE: whitespace; \
            start      -> \"a\" \"b\";\
            whitespace -> \" \";\
";
        let mut peekable = PeekableWrapper::<PeekableWrapper<Chars>>::new(to_parse.chars().peekable());
        let mut vm = NullVm::new();
        let mut parser = RuleParser::new(&mut peekable, &mut vm);
        let _ = parser.parse_rules();
        let start_rule = &parser.parser_data.get_or_add_element_key(&ElementVerbose::new("start".to_string(),ElementType::NonTerminal));
        let whitespace_rule = &parser.parser_data.get_or_add_element_key(&ElementVerbose::new("whitespace".to_string(),ElementType::NonTerminal));
        let rules = &parser.parser_data.parse_rules.rules;

        let start = rules.get(start_rule).unwrap();

        let start_prod = get_elements_or_panic(&parser, start, 0);

        let whitespace = rules.get(whitespace_rule).unwrap();

        let whitespace_prod = get_elements_or_panic(&parser, whitespace, 0);
        assert_eq!(1, start.possible_productions.len());
        assert_eq!(3, start_prod.len());
        assert_eq!(1, whitespace.possible_productions.len());
        assert_eq!(1, whitespace_prod.len());


        assert_eq!(ElementVerbose::new(" ".to_string(),ElementType::Terminal), whitespace_prod[0]);
        assert_eq!(ElementVerbose::new("whitespace".to_string(),ElementType::NonTerminal), start_prod[1]);
    }


    #[test]
    fn test_rule_parser_discarder2() {
        let to_parse =
            "$IGNORE: whitespaces; \
            start      -> \"a\" \"b\" \"c\" ;\
            whitespaces -> whitespace whitespaces_s ;\
            whitespaces_s -> whitespace whitespaces_s| #;\
            whitespace -> \" \";\
";

        let mut peekable = PeekableWrapper::<PeekableWrapper<Chars>>::new(to_parse.chars().peekable());
        let mut vm = NullVm::new();
        let mut parser = RuleParser::new(&mut peekable, &mut vm);
        let _ = &parser.parse_rules().unwrap().rules;
        let start = parser.parser_data.get_rule_by_element_verbose("start").unwrap();
        let start_prod = get_elements_or_panic(&parser, start, 0);
        let whitespaces = parser.parser_data.get_rule_by_element_verbose("whitespaces").unwrap();
        let whitespaces_prod = get_elements_or_panic(&parser, whitespaces, 0);
        let whitespaces_s = parser.parser_data.get_rule_by_element_verbose("whitespaces_s").unwrap();
        let whitespaces_s_prod_0 = get_elements_or_panic(&parser, whitespaces_s, 0);
        let whitespace = parser.parser_data.get_rule_by_element_verbose("whitespace").unwrap();
        let whitespace_prod = get_elements_or_panic(&parser, whitespace, 0);
        assert_eq!(1, start.possible_productions.len());
        assert_eq!(5, start_prod.len());
        assert_eq!(1, whitespaces.possible_productions.len());
        assert_eq!(2, whitespaces_prod.len());
        assert_eq!(2, whitespaces_s.possible_productions.len());
        assert_eq!(2, whitespaces_s_prod_0.len());
        assert_eq!(1, whitespace.possible_productions.len());
        assert_eq!(1, whitespace_prod.len());
    }

    #[test]
    fn test_rule_parser_discarder3() {
        let to_parse =
            "$IGNORE: whitespaces; \
            start      -> \"a\" var ;\
            var -> $[IGNORE:#] \"b\" \"c\";\
            whitespaces -> whitespace whitespaces_s ;\
            whitespaces_s -> whitespace whitespaces_s| #;\
            whitespace -> \" \";\
";

        let mut peekable = PeekableWrapper::<PeekableWrapper<Chars>>::new(to_parse.chars().peekable());
        let mut vm = NullVm::new();
        let mut parser = RuleParser::new(&mut peekable, &mut vm);
        let _ = &parser.parse_rules().unwrap().rules;
        let start = parser.parser_data.get_rule_by_element_verbose("start").unwrap();
        let start_prod = get_elements_or_panic(&parser, start, 0);

        let var = parser.parser_data.get_rule_by_element_verbose("var").unwrap();
        let var_prod = get_elements_or_panic(&parser, var, 0);
        let whitespaces = parser.parser_data.get_rule_by_element_verbose("whitespaces").unwrap();
        let whitespaces_prod = get_elements_or_panic(&parser, whitespaces, 0);
        let whitespaces_s = parser.parser_data.get_rule_by_element_verbose("whitespaces_s").unwrap();
        let whitespaces_s_prod_0 = get_elements_or_panic(&parser, whitespaces_s, 0);
        let whitespace = parser.parser_data.get_rule_by_element_verbose("whitespace").unwrap();
        let whitespace_prod = get_elements_or_panic(&parser, whitespace, 0);
        assert_eq!(1, start.possible_productions.len());
        assert_eq!(3, start_prod.len());
        assert_eq!(1, var.possible_productions.len());
        assert_eq!(2, var_prod.len());
        assert_eq!(1, whitespaces.possible_productions.len());
        assert_eq!(2, whitespaces_prod.len());
        assert_eq!(2, whitespaces_s.possible_productions.len());
        assert_eq!(2, whitespaces_s_prod_0.len());
        assert_eq!(1, whitespace.possible_productions.len());
        assert_eq!(1, whitespace_prod.len());
    }

    #[test]
    fn test_parse_rules_list() {
        let to_parse =
            "start      -> list;\
            list -> l_element list_s ;\
            list_s -> l_element list_s| #;\
            l_element -> \"a\";\
";
        let mut peekable = PeekableWrapper::<PeekableWrapper<Chars>>::new(to_parse.chars().peekable());
        let mut vm = NullVm::new();
        let mut parser = RuleParser::new(&mut peekable, &mut vm);
        let _ = &parser.parse_rules().unwrap().rules;
        let start = parser.parser_data.get_rule_by_element_verbose("start").unwrap();

        let start_prod = get_elements_or_panic(&parser, start, 0);

        let list = parser.parser_data.get_rule_by_element_verbose("list").unwrap();
        let list_s = parser.parser_data.get_rule_by_element_verbose("list_s").unwrap();
        let l_element = parser.parser_data.get_rule_by_element_verbose("l_element").unwrap();

        let list_prod = get_elements_or_panic(&parser, list, 0);
        let list_s_prod = get_elements_or_panic(&parser, list_s, 0);
        let l_element_prod = get_elements_or_panic(&parser, l_element, 0);
        assert_eq!(1, start.possible_productions.len());
        assert_eq!(1, list.possible_productions.len());
        assert_eq!(2, list_s.possible_productions.len());
        assert_eq!(1, l_element.possible_productions.len());
        assert_eq!(1, start_prod.len());
        assert_eq!(2, list_prod.len());

        assert_eq!(ElementVerbose::new("list".to_string(),ElementType::NonTerminal), start_prod[0]);
        assert_eq!(ElementVerbose::new("l_element".to_string(),ElementType::NonTerminal), list_prod[0]);
        assert_eq!(ElementVerbose::new("list_s".to_string(),ElementType::NonTerminal), list_prod[1]);
        assert_eq!(ElementVerbose::new("l_element".to_string(),ElementType::NonTerminal), list_s_prod[0]);
        assert_eq!(ElementVerbose::new("list_s".to_string(),ElementType::NonTerminal), list_s_prod[1]);
        assert_eq!(ElementVerbose::new("a".to_string(),ElementType::Terminal), l_element_prod[0]);
    }

    #[test]
    fn test_is_derivative() {
        let to_parse =
            "start      -> list;\
            list -> l_element list_s ;\
            list_s -> l_element list_s| #;\
            l_element -> \"a\";\
";
        let mut peekable = PeekableWrapper::<PeekableWrapper<Chars>>::new(to_parse.chars().peekable());
        let mut vm = NullVm::new();
        let mut parser = RuleParser::new(&mut peekable, &mut vm);
        let _ = parser.parse_rules();
        let rules = &parser.parser_data.parse_rules;
        let elements= &parser.parser_data.element_types;



        assert!(check_is_derivative(elements,rules, parser.parser_data.get_element_nt_index("start").unwrap(), parser.parser_data.get_element_nt_index("list").unwrap()).unwrap());
        assert!(check_is_derivative(elements,rules, parser.parser_data.get_element_nt_index("start").unwrap(), parser.parser_data.get_element_nt_index("list_s").unwrap()).unwrap());
        assert!(check_is_derivative(elements,rules, parser.parser_data.get_element_nt_index("start").unwrap(), parser.parser_data.get_element_nt_index("l_element").unwrap()).unwrap());
        assert!(check_is_derivative(elements,rules, parser.parser_data.get_element_nt_index("list").unwrap(), parser.parser_data.get_element_nt_index("list_s").unwrap()).unwrap());
        assert!(check_is_derivative(elements,rules, parser.parser_data.get_element_nt_index("list").unwrap(), parser.parser_data.get_element_nt_index("l_element").unwrap()).unwrap());
        assert!(!check_is_derivative(elements,rules, parser.parser_data.get_element_nt_index("l_element").unwrap(), parser.parser_data.get_element_nt_index("start").unwrap()).unwrap());
    }
    #[test]
    fn test_is_derivative1_1() {
        let to_parse =
            "start      -> list;\
            list -> l_element list_s ;\
            list_s -> l_element list_s| #;\
            l_element -> \"a\";\
";
        let mut peekable = PeekableWrapper::<PeekableWrapper<Chars>>::new(to_parse.chars().peekable());
        let mut vm = NullVm::new();
        let mut parser = RuleParser::new(&mut peekable, &mut vm);
        let _ = parser.parse_rules();
        let rules = &parser.parser_data.parse_rules;
        let elements= &parser.parser_data.element_types;
        let mut hasmaotest= HashMap::new();

        hasmaotest.insert("asdf","yoyoyo");
        hasmaotest.insert("234","yoyoyo");
        let start=parser.parser_data.get_element_nt_index("start").unwrap();
        let list_s=parser.parser_data.get_element_nt_index("list_s").unwrap();
        println!("{}",start);
        println!("{}",list_s);
        format!("The origin is: {rules:?}") ;

        assert!(check_is_derivative(elements,rules, start,list_s).unwrap());
  }






    #[test]
    fn test_is_derivative2() {
        let to_parse =
            "start      -> not_end | #;\
            not_end -> \"a\" start ;\
";
        let mut peekable = PeekableWrapper::<PeekableWrapper<Chars>>::new(to_parse.chars().peekable());
        let mut vm = NullVm::new();
        let mut parser = RuleParser::new(&mut peekable, &mut vm);
        let _ = parser.parse_rules();
        let rules = &parser.parser_data.parse_rules;
        let elements= &parser.parser_data.element_types;

        assert!(check_is_derivative(elements,rules, parser.parser_data.get_element_nt_index("start").unwrap(), parser.parser_data.get_element_nt_index("not_end").unwrap()).unwrap());
        assert!(check_is_derivative(elements,rules, parser.parser_data.get_element_nt_index("not_end").unwrap(), parser.parser_data.get_element_nt_index("start").unwrap()).unwrap());
    }
}