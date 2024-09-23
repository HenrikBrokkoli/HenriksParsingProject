use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use parser_data::{ElementData, ElementIndex, ElementType, ParserData};
use vms::{Instruction, VM};

use crate::errors::GrammarError;
use crate::errors::GrammarError::{MissingFollowSet, MissingSteuerSet, SteuerSetsNotDistinct};
use crate::parser_data::{ Production};
use crate::sets::{NamedSets, NamedSetsNoEmpty, SetMember};
use crate::steuer_sets::get_steuer_sets;

/// Forgot what NT means here. NonTerminal?
pub struct NTRules<T> {
    pub steuermap: Steuermap,
    pub ignore: Option<ElementIndex>,
    pub instruction: Option<Box<Instruction<T>>>,
}

pub type Steuermap = HashMap<SetMember, Rc<Production>>;

pub fn get_steuermaps<T>(first_sets: &NamedSets, follow_sets: &NamedSetsNoEmpty, parser_data: ParserData<T>) -> Result<HashMap<ElementIndex, NTRules<T::Tstate>>, GrammarError>
where
    T: VM,
{
    let steuer_sets = get_steuer_sets(first_sets, follow_sets)?;
    let mut steuer_maps = HashMap::new();
    for (rule_name, productions) in parser_data.parse_rules.rules.into_iter() {
        let mut steuermap = Steuermap::new();

        for prod in productions.possible_productions.into_iter() {
            steuermap_of_production(&steuer_sets, follow_sets, &mut steuermap, prod, rule_name, &parser_data.element_types,&parser_data.element_data)?;
        }
        steuer_maps.insert(rule_name, NTRules { steuermap, ignore: productions.ignore, instruction: productions.instruction });
    }
    Ok(steuer_maps)
}

fn steuermap_of_production(steuer_sets: &NamedSetsNoEmpty,
                           follow_sets: &NamedSetsNoEmpty,
                           steuer_map: &mut Steuermap, prod: Rc<Production>,
                           cur_rule_name: ElementIndex,
                           el_types: &Vec<ElementType>,
                            el_data:&Vec<ElementData>)
                           -> Result<(), GrammarError> {
    let prod_ref = Rc::clone(&prod);
    let prod = &*prod;
    match prod {
        Production::NotEmpty(el) => {
            let first = el[0];
            let et = el_types.get(first).ok_or(GrammarError::MissingElementForIndex { index: first })?;
            match et {
                ElementType::Terminal => {
                    let name=el_data.get(first).unwrap().name.clone();
                    let old_key = steuer_map.insert(SetMember::Char(name.chars().next().unwrap()), prod_ref);
                    if old_key.is_some() {
                        return Err(SteuerSetsNotDistinct {
                            steuer_terminal: name.clone(),
                            steuer_char: name.chars().next().unwrap(),
                            rule_name: cur_rule_name.to_string(),
                        });
                    }
                }
                ElementType::NonTerminal => {
                    let steuer_menge = steuer_sets.get(&first).ok_or(MissingSteuerSet { index: first })?;
                    fill_with_steuer_set(steuer_menge, steuer_map, prod_ref, cur_rule_name)?;
                }
            }
            Ok(())
        }
        Production::Empty => {
            let follow_set = follow_sets.get(&cur_rule_name).ok_or(MissingFollowSet { index: cur_rule_name })?;
            fill_with_steuer_set(follow_set, steuer_map, prod_ref, cur_rule_name)?;

            Ok(())
        }
    }
}


fn fill_with_steuer_set(set_no_empty: &HashSet<SetMember>, steuer_map: &mut Steuermap, prod: Rc<Production>, cur_rule_name: ElementIndex) -> Result<(), GrammarError> {
    for follow_char in set_no_empty.iter()
    {
        let old_key = steuer_map.insert(*follow_char, Rc::clone(&prod));
        if old_key.is_some() {
            return Err(SteuerSetsNotDistinct {
                steuer_terminal: String::from("follow_set"),
                steuer_char: match follow_char {
                    SetMember::Char(x) => { *x }
                    SetMember::Terminate => { '#' }
                },
                rule_name: cur_rule_name.to_string(),
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::str::Chars;

    use peekables::PeekableWrapper;
    use vms::NullVm;

    use crate::first_sets::get_first_sets;
    use crate::follow_sets::get_follow_sets;
    use crate::rule_parsing::RuleParser;
    use crate::steuer_map::get_steuermaps;

    //TODO think how to bring this test back

    /*    #[test]
        fn test_steuer_sets() {
            let to_parse =
                "start      -> identifier1
                identifier2;\
                identifier1 -> \"a_terminal\"| #;
                identifier2 -> \"b_terminal\"| #;
    ";
            let mut peekable= PeekableWrapper::<PeekableWrapper<Chars>>::new(to_parse.chars().peekable());
            let vm=NullVm::new();
            let mut rule_parser = RuleParser::new(&mut peekable, &vm);
            rule_parser.parse_rules().unwrap();
            let rule_dict = rule_parser.parse_rules.rules;

            let first_dict = get_first_sets( &rule_dict).unwrap();
            let follow_dict = get_follow_sets("start".to_string(), &rule_dict, &first_dict).unwrap();
            let steuer_maps = get_steuermaps(rule_dict, &first_dict, &follow_dict).unwrap();
            assert!(Rc::ptr_eq(&rule_dict.get("start").unwrap().possible_productions[0], steuer_maps.get("start").unwrap().steuermap.get(&SetMember::Terminate).unwrap()));
            assert!(Rc::ptr_eq(&rule_dict.get("identifier1").unwrap().possible_productions[0], steuer_maps.get("identifier1").unwrap().steuermap.get(&SetMember::Char('a')).unwrap()));
            assert!(Rc::ptr_eq(&rule_dict.get("identifier1").unwrap().possible_productions[1], steuer_maps.get("identifier1").unwrap().steuermap.get(&SetMember::Char('b')).unwrap()));
            assert!(Rc::ptr_eq(&rule_dict.get("identifier1").unwrap().possible_productions[1], steuer_maps.get("identifier1").unwrap().steuermap.get(&SetMember::Terminate).unwrap()));
        }*/

    #[test]
    fn test_1() {
        let to_parse =
            "start -> a|b;\
a -> \"astring\";\
b -> \"bstring\";\
";

        let mut peekable = PeekableWrapper::<PeekableWrapper<Chars>>::new(to_parse.chars().peekable());
        let mut vm = NullVm::new();
        let mut rule_parser = RuleParser::new(&mut peekable, &mut vm);
        let _rules = &rule_parser.parse_rules().unwrap().rules;
        let parser_data = rule_parser.parser_data;
        let first_dict = get_first_sets(&parser_data).unwrap();
        let follow_dict = get_follow_sets(parser_data.get_element_nt_index("start").unwrap(), &first_dict, &parser_data).unwrap();
        let _steuer_maps = get_steuermaps(&first_dict, &follow_dict, parser_data).unwrap();
    }

    #[test]
    fn test_two_with_same_first() {
        let to_parse =
            "start -> a a_a;\
a -> \"a\";\
a_a -> \"a\";\
";

        let mut peekable = PeekableWrapper::<PeekableWrapper<Chars>>::new(to_parse.chars().peekable());
        let mut vm = NullVm::new();
        let rule_parser = RuleParser::new(&mut peekable, &mut vm);
        let parser_data = rule_parser.parser_data;
        let first_dict = get_first_sets(&parser_data).unwrap();
        let follow_dict = get_follow_sets(parser_data.get_element_nt_index("start").unwrap(), &first_dict, &parser_data).unwrap();
        let _steuer_maps = get_steuermaps(&first_dict, &follow_dict, parser_data).unwrap();
    }

    #[test]
    fn test_steuer_map_list() {
        let rules =
            "start      -> list ;\
            list -> element list_s ;\
            list_s -> element list_s| #;\
            element -> \"a\";\
";

        let mut peekable = PeekableWrapper::<PeekableWrapper<Chars>>::new(rules.chars().peekable());
        let mut vm = NullVm::new();
        let rule_parser = RuleParser::new(&mut peekable, &mut vm);
        let parser_data = rule_parser.parser_data;
        let first_dict = get_first_sets(&parser_data).unwrap();
        let follow_dict = get_follow_sets(parser_data.get_element_nt_index("start").unwrap(), &first_dict, &parser_data).unwrap();
        let _steuer_maps = get_steuermaps(&first_dict, &follow_dict, parser_data).unwrap();
    }


    #[test]
    fn test_more_whitespace_shenanigans() {
        let to_parse =
            "$IGNORE: whitespaces;\
start -> zero zero;\
\
terms -> term terms_s;\
terms_s -> term terms_s | #;\
term -> zero;\
zero -> \"0\";\
\
whitespaces -> whitespace whitespaces_s;\
whitespaces_s -> whitespace whitespaces_s| #;\
whitespace -> \" \";";

        let mut peekable = PeekableWrapper::<PeekableWrapper<Chars>>::new(to_parse.chars().peekable());
        let mut vm = NullVm::new();
        let rule_parser = RuleParser::new(&mut peekable, &mut vm);
        let parser_data = rule_parser.parser_data;
        let first_dict = get_first_sets(&parser_data).unwrap();
        let follow_dict = get_follow_sets(parser_data.get_element_nt_index("start").unwrap(), &first_dict, &parser_data).unwrap();
        let _steuer_maps = get_steuermaps(&first_dict, &follow_dict, parser_data).unwrap();
    }

    #[test]
    fn test_more_whitespace_shenanigans3() {
        let to_parse =
            "$IGNORE: whitespaces;\
start  -> terms;\
\
terms -> term terms_s;\
terms_s -> term terms_s | #;\
\
\
term -> number;\
\
\
number-> $[IGNORE:#] zero number_s;\
number_s -> number_s_ | #;\
number_s_ -> $[IGNORE:#] zero number_s;\
zero ->  \"0\";\
\
whitespaces -> whitespace whitespaces_s;\
whitespaces_s -> whitespace whitespaces_s| #;\
whitespace -> \" \";";

        let mut peekable = PeekableWrapper::<PeekableWrapper<Chars>>::new(to_parse.chars().peekable());
        let mut vm = NullVm::new();
        let rule_parser = RuleParser::new(&mut peekable, &mut vm);
        let parser_data = rule_parser.parser_data;
        let first_dict = get_first_sets(&parser_data).unwrap();
        let follow_dict = get_follow_sets(parser_data.get_element_nt_index("start").unwrap(), &first_dict, &parser_data).unwrap();
        let _steuer_maps = get_steuermaps(&first_dict, &follow_dict, parser_data).unwrap();
    }
}