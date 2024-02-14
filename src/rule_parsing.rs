use std::borrow::Borrow;
use std::collections::{HashMap, HashSet};
use std::mem;
use std::rc::Rc;
use std::str::Chars;

use vms::{Instruction, VM};
use crate::errors::GrammarError::UnexpectedElementError;
use crate::errors::ParserError;
use crate::errors::ParserError::{EndOfCharsError, UnexpectedCharError};
use crate::parse_funcs::{parse_var_name, parse_symbol, parse_whitespace};
use crate::peekables::{ParseProcess, PeekableWrapper, TPeekable};
//use crate::virtual_machine::{VirtualMachine};


pub struct ParseRules<'vm, T> where T: VM {
    pub rules: RuleMap<'vm, T>,
    pub ignore: Option<String>,
}

pub type RuleMap<'vm, T> = HashMap<String, NonTerminalRules<'vm, T>>;


pub type Ppp<'pp> = ParseProcess<'pp, PeekableWrapper<Chars<'pp>>>;

pub struct NonTerminalRules<'vm, T> where T: VM {
    pub vm: &'vm T,
    pub possible_productions: PossibleProductions,
    pub ignore: Option<String>,
    pub instruction: Option<Box<Instruction<T::Tstate>>>,
}

pub type PossibleProductions = Vec<Rc<Production>>;

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum ET {
    Terminal(String),
    NonTerminal(String),
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Element {
    pub el_type: ET,
    pub keep_data: bool,
}

impl Element {
    pub fn new(et_type: ET, keep_data: bool) -> Element {
        Element { el_type: et_type, keep_data }
    }
}

pub enum Production {
    NotEmpty(Vec<Element>),
    Empty,
}

// pub struct RuleParser<'vm, 'pp, T> where T: VM {
//     peekable_wrapper: PeekableWrapper<Chars<'pp>>,
//     rule_parser: RuleParser_<'vm, 'pp, T>,
// }
//
// impl<'vm, 'pp, T> RuleParser<'vm, 'pp, T> where T: VM {
//     pub fn new(to_parse: &'pp str, vm: &'vm T) -> RuleParser<'vm, 'pp, T> {
//         let mut peekable = PeekableWrapper::<Chars<'pp>>::new(to_parse.chars().peekable());
//         RuleParser { peekable_wrapper: peekable, rule_parser: RuleParser_::new(&mut peekable, vm) }
//     }
//     pub fn parse_rules(&mut self) -> Result<&ParseRules<'vm, T>, ParserError> { self.rule_parser.parse_rules() }
//     pub fn parse_rule(&mut self) -> Result<(String, NonTerminalRules<'vm, T>), ParserError> { self.rule_parser.parse_rule() }
// }

pub struct RuleParser<'vm, 'pp, T> where T: VM {
    pub vm: &'vm T,
    pub parse_process: Ppp<'pp>,
    pub parse_rules: ParseRules<'vm, T>,
}

impl<'vm, 'pp, T> RuleParser<'vm, 'pp, T> where T: VM {
    pub fn new(peekable: &'pp mut PeekableWrapper<Chars<'pp>>, vm: &'vm T) -> RuleParser<'vm, 'pp, T> {
        let parse_process = ParseProcess::<PeekableWrapper<Chars>>::new(peekable, None, None);
        RuleParser {
            vm,
            parse_process,
            parse_rules: ParseRules { rules: Default::default(), ignore: None },
        }
    }

    pub fn parse_rules(&mut self) -> Result<&ParseRules<'vm, T>, ParserError> {
        self.parse_special()?;
        loop {
            parse_whitespace(&mut self.parse_process);
            if self.parse_process.peek().is_none() {
                break;
            }
            let (rule_name, rule) = self.parse_rule()?;
            self.merge_rule(rule, &rule_name);
        }
        self.edit_rules()?;
        Ok(&self.parse_rules)
    }

    fn parse_special(&mut self) -> Result<(), ParserError> {
        if parse_symbol(&mut self.parse_process, '$').is_ok() {
            let special_instruction = parse_var_name(&mut self.parse_process)?;
            parse_symbol(&mut self.parse_process, ':')?;
            parse_whitespace(&mut self.parse_process);
            if special_instruction == "IGNORE" {
                if parse_symbol(&mut self.parse_process, '#').is_ok() {
                    self.parse_rules.ignore = None
                } else {
                    self.parse_rules.ignore = Some(parse_var_name(&mut self.parse_process)?);
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

        //First we iterate over the rules and see what actions o do. Then we take all rules iterate a second time to do the actions on them

        //collect actions to do on the rules in a hashmap
        let mut actions = HashMap::new();
        for (rule_name, rule) in self.parse_rules.rules.iter() {
            let mut weave_in: bool = false;
            if let Some(ignore) = &rule.ignore {
                if !check_is_derivative(&self.parse_rules, ignore, rule_name)? {
                    weave_in = true;
                }
            }
            let action = Action { weave_in };
            actions.insert(rule_name.clone(), action);
        }

        //take rules iterate over them and edit them.
        let rules_to_edit = mem::take(&mut self.parse_rules.rules);
        for (rule_name, rule) in rules_to_edit.into_iter() {
            let r_name=rule_name.clone();
            let action = actions.get(&rule_name).ok_or(ParserError::InternalError { message: format!("keine Action vorhandne für {rule_name}") })?;//should never happen because in the first iteration we made an entry for every rule
            let possible_productions: PossibleProductions;
            let mut ignore_new = rule.ignore.clone();
            if let Some(ignore) = &rule.ignore {
                if action.weave_in {
                    possible_productions = self.weave_in_ignorers(&rule, ignore);
                } else {
                    ignore_new = None;
                    possible_productions = rule.possible_productions;
                }
            } else {
                possible_productions = rule.possible_productions;
            }
            let new_rul = NonTerminalRules::<'vm, T> { vm: rule.vm, possible_productions, ignore: ignore_new, instruction: rule.instruction };

            edited_rules.insert(rule_name.clone(), new_rul);
        }

        //put rules back
        self.parse_rules.rules = edited_rules;
        Ok(())
    }
    fn merge_rule(&mut self, mut rule: NonTerminalRules<'vm, T>, rule_name: &str) {
        if let Some(rule_to_change) = self.parse_rules.rules.get_mut(rule_name) {
            rule_to_change.possible_productions.append(&mut rule.possible_productions)
        } else {
            self.parse_rules.rules.insert(String::from(rule_name), rule);
        }
    }
    pub fn parse_rule(&mut self) -> Result<(String, NonTerminalRules<'vm, T>), ParserError> {
        let identifier = parse_var_name(&mut self.parse_process)?;
        parse_whitespace(&mut self.parse_process);
        parse_symbol(&mut self.parse_process, '-')?;
        parse_symbol(&mut self.parse_process, '>')?;
        parse_whitespace(&mut self.parse_process);
        let ignore_this_maybe = self.parse_overrides()?;
        let productions: PossibleProductions = parse_possible_productions(&mut self.parse_process)?;
        let instruction = self.parse_instruction_section(&identifier)?;
        parse_whitespace(&mut self.parse_process);
        parse_symbol(&mut self.parse_process, ';')?;
        let nt_rules = NonTerminalRules::<'vm, T> { vm: self.vm, possible_productions: productions, ignore: ignore_this_maybe, instruction };
        Ok((identifier, nt_rules))
    }


    fn weave_in_ignorers(&self, rule: &NonTerminalRules<'vm, T>, ignore: &str) -> Vec<Rc<Production>> {
        let mut weaved_productions = vec![];
        for production in rule.possible_productions.iter() {
            let weaved_production = Rc::new(self.weave_in_ignorers_single_production(production.clone(), Element::new(ET::NonTerminal((ignore).to_owned()), false)));
            weaved_productions.push(weaved_production);
        }


        weaved_productions
    }

    fn weave_in_ignorers_single_production(&self, production: Rc<Production>, to_weave_in: Element) -> Production {
        match production.borrow() {
            Production::NotEmpty(prod) => {
                let mut weaved_production = vec![];
                //weaved_production.push(to_weave_in.clone()); cant insert here or two weaved ignorers will be next to each other and that will lead to bugs
                for el in prod {
                    weaved_production.push(el.clone());
                    weaved_production.push(to_weave_in.clone());
                }
                weaved_production.pop();


                Production::NotEmpty(weaved_production)
            }
            Production::Empty => { Production::Empty }//cant add weave in here or bugs
        }
    }

    fn parse_overrides(&mut self) -> Result<Option<String>, ParserError> {
        let mut ignore_this = self.parse_rules.ignore.clone();
        if parse_symbol(&mut self.parse_process, '$').is_ok() {
            parse_symbol(&mut self.parse_process, '[')?;
            let varname = parse_var_name(&mut self.parse_process)?;
            parse_whitespace(&mut self.parse_process);
            parse_symbol(&mut self.parse_process, ':')?;


            if varname == "IGNORE" {
                if parse_symbol(&mut self.parse_process, '#').is_ok() {
                    ignore_this = None;
                } else {
                    ignore_this = Some(parse_var_name(&mut self.parse_process)?);
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
}

fn check_is_derivative<T>(parse_rules: &ParseRules<T>, left: &String, right: &String) -> Result<bool, ParserError> where T: VM {
    _check_is_derivative(parse_rules, left, right, &mut HashSet::new())
}

fn _check_is_derivative<T>(parse_rules: &ParseRules<T>, left: &String, right: &String, visited: &mut HashSet<String>) -> Result<bool, ParserError> where T: VM {
    let prods = &parse_rules.rules.get(left).ok_or(ParserError::InternalError { message: format!("could not find {left}") })?.possible_productions;

    if left == right {
        return Ok(true);
    }
    if visited.contains(left) {
        return Ok(false);
    }
    visited.insert(left.clone());

    let mut is_der = false;
    for prod in prods {
        match &**prod {
            Production::NotEmpty(prod_ne, ..) => {
                for el in prod_ne {
                    match &el.el_type {
                        ET::Terminal(_) => {}
                        ET::NonTerminal(el) => {
                            if el == right {
                                return Ok(true);
                            } else {
                                is_der = is_der || _check_is_derivative(parse_rules, el, right, visited)?;
                            }
                        }
                    }
                }
            }
            Production::Empty => {}
        }
    }
    Ok(is_der)
}


fn parse_possible_productions(parse_process: &mut ParseProcess<PeekableWrapper<Chars>>) -> Result<PossibleProductions, ParserError> {
    let mut elements = vec![parse_production(parse_process)?];
    loop {
        parse_whitespace(parse_process);
        let _ = parse_symbol(parse_process, '\n');
        parse_whitespace(parse_process);
        if parse_symbol(parse_process, '|').is_ok() {
            parse_whitespace(parse_process);
            elements.push(parse_production(parse_process)?);
        } else {
            break;
        }
    }
    Ok(elements)
}

fn parse_production(parse_process: &mut ParseProcess<PeekableWrapper<Chars>>) -> Result<Rc<Production>, ParserError> {
    let mut result = vec![];
    loop {
        parse_whitespace(parse_process);
        if let Ok(element) = parse_element(parse_process) {
            match element {
                None => {
                    if !result.is_empty() {
                        return Err(ParserError::GramError { err: UnexpectedElementError { reason: String::from("Leereelemente für leere Menge darf nicht mit anderen Elementen zusammen stehen"), pos: parse_process.cur_pos() } });
                    }
                    return Ok(Rc::new(Production::Empty));
                }
                Some(el) => {
                    match el.el_type {
                        ET::Terminal(term) => { result.push(Element::new(ET::Terminal(term), true)) }
                        ET::NonTerminal(nonterm) => { result.push(Element::new(ET::NonTerminal(nonterm), true)) }
                    }
                }
            }
        } else {
            break;
        }
    }

    if !result.is_empty() {
        return Ok(Rc::new(Production::NotEmpty(result)));
    }
    Err(ParserError::GramError { err: UnexpectedElementError { reason: String::from("Es wurden keine Elemente gefunden. Es müssen aber welche gefunden werden oder es muss das Symbol für leere Menge genutzt werden (#)"), pos: parse_process.cur_pos() } })
}

fn parse_element<T>(to_parse: &mut ParseProcess<T>) -> Result<Option<Element>, ParserError> where T: TPeekable<Item=char> {
    match to_parse.peek() {
        Some('#') => {
            to_parse.next();
            Ok(None)
        }
        Some('"') => Ok(Some(Element::new(ET::Terminal(parse_terminal(to_parse)?), true))),
        Some(x) if x.is_alphabetic() => Ok(Some(Element::new(ET::NonTerminal(parse_var_name(to_parse)?), true))),
        Some(x) => Err(UnexpectedCharError { chr: *x, pos: to_parse.cur_pos(),expected:String::from("char # for empty, \" for terminal ort alphabetic for element") }),
        _ => Err(EndOfCharsError)
    }
}

pub fn parse_terminal<T>(to_parse: &mut ParseProcess<T>) -> Result<String, ParserError> where T: TPeekable<Item=char> {
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
        let mut to_parse = ParseProcess::<PeekableWrapper<Chars>>::new(&mut peekable, None, None);
        let result = &*parse_production(&mut to_parse).unwrap();
        let result = match result {
            Production::Empty => { panic!() }
            Production::NotEmpty(x, ..) => x
        };

        assert_eq!(Element::new(ET::NonTerminal("identifier".to_string()), true), *result.get(0).unwrap());
        assert_eq!(Element::new(ET::Terminal("terminal".to_string()), true), *result.get(1).unwrap());
        assert_eq!(Element::new(ET::NonTerminal("identifier2".to_string()), true), *result.get(2).unwrap());
        assert_eq!(Element::new(ET::Terminal("terminal2".to_string()), true), *result.get(3).unwrap());
    }

    #[test]
    fn test_parse_production2() {
        let mut peekable = PeekableWrapper::<Chars>::new("identifier \"terminal\"
        |   identifier2 \"terminal\"".chars().peekable());
        let mut to_parse =
            ParseProcess::<PeekableWrapper<Chars>>::new(&mut peekable, None, None);
        let result = parse_possible_productions(&mut to_parse).unwrap();
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

        assert_eq!(Element::new(ET::NonTerminal("identifier".to_string()), true), *result_first.get(0).unwrap());
        assert_eq!(Element::new(ET::Terminal("terminal".to_string()), true), *result_first.get(1).unwrap());
        assert_eq!(Element::new(ET::NonTerminal("identifier2".to_string()), true), *result_second.get(0).unwrap());
        assert_eq!(Element::new(ET::Terminal("terminal".to_string()), true), *result_second.get(1).unwrap());
    }

    #[test]
    fn test_parse_rule() {
        let to_parse = "identifier ->  identifier2;";
        let mut peekable = PeekableWrapper::<PeekableWrapper<Chars>>::new(to_parse.chars().peekable());
        let vm = NullVm::new();
        let mut parser = RuleParser::new(&mut peekable, &vm);
        let (rule_name, rule) = parser.parse_rule().unwrap();
        assert_eq!("identifier", rule_name);
        let first = &**rule.possible_productions.get(0).unwrap();
        let result = match first {
            Production::Empty => { panic!() }
            Production::NotEmpty(x, ..) => x
        };
        assert_eq!(Element::new(ET::NonTerminal("identifier2".to_string()), true), *result.get(0).unwrap());
    }

    #[test]
    fn test_parse_rules() {
        let to_parse =
            "rule1      -> rule2
               |rule3;
rule2 -> \"b_terminal\"\
               | \"c_terminal\";";
        let mut peekable = PeekableWrapper::<PeekableWrapper<Chars>>::new(to_parse.chars().peekable());
        let  vm = NullVm::new();
        let mut parser = RuleParser::new(&mut peekable, & vm);
        let rules = &parser.parse_rules().unwrap().rules;
        let rule1 = rules.get("rule1").unwrap();
        let rule2 = rules.get("rule2").unwrap();
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
        assert_eq!(Element::new(ET::NonTerminal("rule2".to_string()), true), *rule1_production1.get(0).unwrap());
        assert_eq!(Element::new(ET::NonTerminal("rule3".to_string()), true), *rule1_production2.get(0).unwrap());
        assert_eq!(Element::new(ET::Terminal("b_terminal".to_string()), true), *rule2_production1.get(0).unwrap());
        assert_eq!(Element::new(ET::Terminal("c_terminal".to_string()), true), *rule2_production2.get(0).unwrap());
    }

    #[test]
    #[should_panic]
    fn test_parse_rules2() {
        let  to_parse =
            "rule1      -> rule2
               |rule3;
rule2 -> \"b_terminal\"\
               | \"c_terminal\"; asdf";

        let mut peekable = PeekableWrapper::<PeekableWrapper<Chars>>::new(to_parse.chars().peekable());
        let  vm = NullVm::new();
        let mut parser = RuleParser::new(&mut peekable, & vm);
        let _rules = &parser.parse_rules().unwrap().rules;
    }


    #[test]
    fn test_parse_rules3() {
        let  to_parse =
            "start ->   identifier2
                        |identifier3
                        |\"a_terminal\";\
            identifier2 -> \"b_terminal\"\
                            | #;
            identifier3 -> \"c_terminal\";
";
        let mut peekable = PeekableWrapper::<PeekableWrapper<Chars>>::new(to_parse.chars().peekable());
        let  vm = NullVm::new();
        let mut parser = RuleParser::new(&mut peekable, & vm);
        let _rules = &parser.parse_rules().unwrap().rules;
    }


    fn get_elements_or_panic<'a, T>(rules: &'a NonTerminalRules<'_, T>, index: usize) -> &'a Vec<Element> where T: VM {
        let production = &**rules.possible_productions.get(index).unwrap();
        match production {
            Production::Empty => { panic!() }
            Production::NotEmpty(x, ..) => x
        }
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
        let  vm = NullVm::new();
        let mut parser = RuleParser::new(&mut peekable, & vm);
        let rules = &parser.parse_rules().unwrap().rules;
        let rule1 = rules.get("rule1").unwrap();
        let rule2 = rules.get("rule2").unwrap();
        let rule1_production1 = get_elements_or_panic(rule1, 0);
        let rule1_production2 = get_elements_or_panic(rule1, 1);
        let rule2_production1 = get_elements_or_panic(rule2, 0);
        let rule2_production2 = get_elements_or_panic(rule2, 1);
        assert_eq!(Element::new(ET::NonTerminal("rule2".to_string()), true), rule1_production1[0]);
        assert_eq!(Element::new(ET::NonTerminal("rule3".to_string()), true), rule1_production2[0]);
        assert_eq!(Element::new(ET::Terminal("b_terminal".to_string()), true), rule2_production1[0]);
        assert_eq!(Element::new(ET::Terminal("c_terminal".to_string()), true), rule2_production2[0]);
    }

    #[test]
    fn test_parse_rules5() {
        let to_parse =
            "start      -> not_end | #;\
            not_end -> \"a\" start ;\
";
        let mut peekable = PeekableWrapper::<PeekableWrapper<Chars>>::new(to_parse.chars().peekable());
        let  vm = NullVm::new();
        let mut parser = RuleParser::new(&mut peekable, & vm);
        let rules = &parser.parse_rules().unwrap().rules;
        let start = rules.get("start").unwrap();
        let not_end = rules.get("not_end").unwrap();
        let start_prod = get_elements_or_panic(start, 0);
        let not_end_prod = get_elements_or_panic(not_end, 0);
        assert_eq!(2, start.possible_productions.len());
        assert_eq!(1, not_end.possible_productions.len());

        assert_eq!(Element::new(ET::NonTerminal("not_end".to_string()), true), start_prod[0]);
        assert_eq!(Element::new(ET::NonTerminal("start".to_string()), true), not_end_prod[1]);
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
        let rules = &parser.parse_rules().unwrap().rules;
        let start = rules.get("start").unwrap();

        let start_prod = get_elements_or_panic(start, 0);

        let whitespace = rules.get("whitespace").unwrap();

        let whitespace_prod = get_elements_or_panic(whitespace, 0);
        assert_eq!(1, start.possible_productions.len());
        assert_eq!(3, start_prod.len());
        assert_eq!(1, whitespace.possible_productions.len());
        assert_eq!(1, whitespace_prod.len());


        assert_eq!(Element::new(ET::Terminal(" ".to_string()), true), whitespace_prod[0]);
        assert_eq!(Element::new(ET::NonTerminal("whitespace".to_string()), false), start_prod[1]);
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
        let rules = &parser.parse_rules().unwrap().rules;
        let start = rules.get("start").unwrap();
        let start_prod = get_elements_or_panic(start, 0);
        let whitespaces = rules.get("whitespaces").unwrap();
        let whitespaces_prod = get_elements_or_panic(whitespaces, 0);
        let whitespaces_s = rules.get("whitespaces_s").unwrap();
        let whitespaces_s_prod_0 = get_elements_or_panic(whitespaces_s, 0);
        let whitespace = rules.get("whitespace").unwrap();
        let whitespace_prod = get_elements_or_panic(whitespace, 0);
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
        let rules = &parser.parse_rules().unwrap().rules;
        let start = rules.get("start").unwrap();
        let start_prod = get_elements_or_panic(start, 0);

        let var = rules.get("var").unwrap();
        let var_prod = get_elements_or_panic(var, 0);
        let whitespaces = rules.get("whitespaces").unwrap();
        let whitespaces_prod = get_elements_or_panic(whitespaces, 0);
        let whitespaces_s = rules.get("whitespaces_s").unwrap();
        let whitespaces_s_prod_0 = get_elements_or_panic(whitespaces_s, 0);
        let whitespace = rules.get("whitespace").unwrap();
        let whitespace_prod = get_elements_or_panic(whitespace, 0);
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
        let rules = &parser.parse_rules().unwrap().rules;
        let start = rules.get("start").unwrap();

        let start_prod = get_elements_or_panic(start, 0);

        let list = rules.get("list").unwrap();
        let list_s = rules.get("list_s").unwrap();
        let l_element = rules.get("l_element").unwrap();

        let list_prod = get_elements_or_panic(list, 0);
        let list_s_prod = get_elements_or_panic(list_s, 0);
        let l_element_prod = get_elements_or_panic(l_element, 0);
        assert_eq!(1, start.possible_productions.len());
        assert_eq!(1, list.possible_productions.len());
        assert_eq!(2, list_s.possible_productions.len());
        assert_eq!(1, l_element.possible_productions.len());
        assert_eq!(1, start_prod.len());
        assert_eq!(2, list_prod.len());

        assert_eq!(Element::new(ET::NonTerminal("list".to_string()), true), start_prod[0]);
        assert_eq!(Element::new(ET::NonTerminal("l_element".to_string()), true), list_prod[0]);
        assert_eq!(Element::new(ET::NonTerminal("list_s".to_string()), true), list_prod[1]);
        assert_eq!(Element::new(ET::NonTerminal("l_element".to_string()), true), list_s_prod[0]);
        assert_eq!(Element::new(ET::NonTerminal("list_s".to_string()), true), list_s_prod[1]);
        assert_eq!(Element::new(ET::Terminal("a".to_string()), true), l_element_prod[0]);
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
        let rules = &parser.parse_rules().unwrap();

        assert!(check_is_derivative(rules, &"start".to_string(), &"list".to_string()).unwrap());
        assert!(check_is_derivative(rules, &"start".to_string(), &"list_s".to_string()).unwrap());
        assert!(check_is_derivative(rules, &"start".to_string(), &"l_element".to_string()).unwrap());
        assert!(check_is_derivative(rules, &"list".to_string(), &"list_s".to_string()).unwrap());
        assert!(check_is_derivative(rules, &"list".to_string(), &"l_element".to_string()).unwrap());
        assert!(!check_is_derivative(rules, &"l_element".to_string(), &"start".to_string()).unwrap());
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
        let rules = &parser.parse_rules().unwrap();

        assert!(check_is_derivative(rules, &"start".to_string(), &"not_end".to_string()).unwrap());
        assert!(check_is_derivative(rules, &"not_end".to_string(), &"start".to_string()).unwrap());
    }


}