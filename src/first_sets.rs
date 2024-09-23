use crate::errors::GrammarError;
use parser_data::{ElementIndex, ElementType, ParserData, Production};
use peekables::PeekableWrapper;
use std::collections::{HashMap, HashSet};
use std::str::Chars;

use crate::sets::{NamedSets, SetMemberWithEmpty};
use crate::vms::{NullVm, VM};

///Computes the first set of a slice of elements. Usually used to get the first set of a partial right side of a production.
/// # Arguments
/// * `elements` a reference to a slice of elements. The first_set is computed on the basis that the elements are following in this order
/// * `first_sets` a named Set of first_sets. For every element in elements there has to be an entry in first_sets
///
/// For example, we have the production a -> element1 element2 element3 and want to start computing the follow_set of element1
/// For that we need the first_set of [element2, element3].
/// Our first_sets are {element2: ['a',empty], element3: ['b','c']}
/// Our return  will be {'a','b','c'}
pub fn first_set_of_partial<T>(el_ixs: &[ElementIndex], first_sets: &NamedSets, parser_data: &ParserData<T>) -> Result<HashSet<SetMemberWithEmpty>, GrammarError> where T: VM {
    let mut set = HashSet::new();

    for &ix in el_ixs {
        let cur_set = get_first_set_of_element(ix, first_sets, parser_data)?
            .ok_or(GrammarError::MissingFirstSet { index: ix })?;
        let has_empty = cur_set.contains(&SetMemberWithEmpty::Empty);
        set.extend(cur_set);
        if !has_empty {
            set.remove(&SetMemberWithEmpty::Empty);
            break;
        }
    }
    Ok(set)
}


pub fn get_first_sets<T>(parser_data: &ParserData<T>) -> Result<NamedSets, GrammarError> where T: VM {
    let mut first_sets: NamedSets = HashMap::new();
    for &el_ix in parser_data.parse_rules.rules.keys() {
        get_set_first_set_of_element(el_ix, &mut first_sets, parser_data)?;
    }
    Ok(first_sets)
}


fn get_set_first_set_of_element<T>(el_ix: ElementIndex, first_sets: &mut NamedSets, parser_data: &ParserData<T>) -> Result<HashSet<SetMemberWithEmpty>, GrammarError> where T: VM {
    let set_maybe = get_first_set_of_element(el_ix, first_sets, parser_data)?;
    let res=match set_maybe {
        None => {
            let mut set: HashSet<SetMemberWithEmpty> = HashSet::new();
            let productions = match parser_data.parse_rules.rules.get(&el_ix) {
                None => { return Err(GrammarError::MissingProduction { index: el_ix }); }
                Some(x) => x
            };
            for production in productions.possible_productions.iter() {
                let rus=get_first_set_of_production(production, first_sets, parser_data)?;
                set.extend(&rus)
            }
            first_sets.insert(el_ix, set.clone());
            Ok(set)
        }
        Some(x) => Ok(x)
    };
    res
}


fn get_first_set_of_element<T>(el_ix: ElementIndex, first_sets: &NamedSets, parser_data: &ParserData<T>) -> Result<Option<HashSet<SetMemberWithEmpty>>, GrammarError> where T: VM {
    let et= parser_data.element_types[el_ix];
    let res = match et {
        ElementType::NonTerminal => {
            first_sets.get(&el_ix).map(|entry| (*entry).clone())
        }
        ElementType::Terminal => {
            let el_verb=&parser_data.get_element_data(el_ix).unwrap().name;
            let maybe_char = el_verb.chars().next();
            Some(match maybe_char {
                None => { HashSet::from([SetMemberWithEmpty::Empty]) }
                Some(ch) => HashSet::from([SetMemberWithEmpty::Char(ch)])
            })
        }
    };
    Ok(res)
}


fn get_first_set_of_production<T>(production: &Production, first_sets: &mut NamedSets, parser_data: &ParserData<T>) -> Result<HashSet<SetMemberWithEmpty>, GrammarError> where T: VM {
    match production {
        Production::Empty => Ok(HashSet::from([SetMemberWithEmpty::Empty])),
        Production::NotEmpty(el_ixs) => {
            let mut first_menge = get_set_first_set_of_element(*el_ixs.first().unwrap(), first_sets, parser_data)?;
            if first_menge.contains(&SetMemberWithEmpty::Empty) {
                for el_ix in el_ixs.iter().skip(1) {
                    let cur_set = get_set_first_set_of_element(*el_ix, first_sets, parser_data)?;
                    let has_empty = cur_set.contains(&SetMemberWithEmpty::Empty);
                    first_menge.extend(cur_set);
                    if !has_empty {
                        first_menge.remove(&SetMemberWithEmpty::Empty);
                        break;
                    }
                }
            }
            Ok(first_menge)
        }
    }
}


#[cfg(test)]
mod tests {
    use peekables::PeekableWrapper;
    use std::str::Chars;
    use vms::NullVm;

    use crate::rule_parsing::RuleParser;
    use crate::test_helpers::make_memberset;

    use super::*;

    #[test]
    fn test_first_mengen() {
        let to_parse =
            "start ->   identifier2
                        |identifier3
                        |\"a_terminal\";\
            identifier2 -> \"b_terminal\"\
                            | #;
            identifier3 -> \"c_terminal\";
";
        let mut peekable = PeekableWrapper::<PeekableWrapper<Chars>>::new(to_parse.chars().peekable());
        let mut vm = NullVm::new();
        let mut rule_parser = RuleParser::new(&mut peekable, &mut vm);
        let _ = rule_parser.parse_rules().unwrap().rules;
        let first_dict = get_first_sets(&rule_parser.parser_data).unwrap();
        assert_eq!(make_memberset("abc#"), first_dict.get(&rule_parser.parser_data.get_element_nt_index("start").unwrap()).unwrap().clone());
        assert_eq!(make_memberset("b#"), first_dict.get(&rule_parser.parser_data.get_element_nt_index("identifier2").unwrap()).unwrap().clone());
        assert_eq!(make_memberset("c"), first_dict.get(&rule_parser.parser_data.get_element_nt_index("identifier3").unwrap()).unwrap().clone());
        assert_eq!(make_memberset("b"), first_dict.get(&rule_parser.parser_data.get_element_nt_index("b_terminal").unwrap()).unwrap().clone())
    }


    #[test]
    fn test_first_mengen2() {
        let to_parse =
            "start      ->  identifier2 identifier3
                            | \"a_terminal\";\
            identifier2 ->  \"b_terminal\"\
                            | #;
            identifier3 -> \"c_terminal\";
";
        let mut peekable = PeekableWrapper::<PeekableWrapper<Chars>>::new(to_parse.chars().peekable());
        let mut vm = NullVm::new();
        let mut rule_parser = RuleParser::new(&mut peekable, &mut vm);
        let _ = rule_parser.parse_rules().unwrap().rules;
        let first_dict = get_first_sets(&rule_parser.parser_data).unwrap();
        assert_eq!(make_memberset("b#"), first_dict.get(&rule_parser.parser_data.get_element_nt_index("identifier2").unwrap()).unwrap().clone());
        assert_eq!(make_memberset("c"), first_dict.get(&rule_parser.parser_data.get_element_nt_index("identifier3").unwrap()).unwrap().clone());
        assert_eq!(make_memberset("abc"), first_dict.get(&rule_parser.parser_data.get_element_nt_index("start").unwrap()).unwrap().clone());
    }

    #[test]
    fn test_first_mengen3() {
        let to_parse =
            "start      ->  identifier2 identifier3
                            | \"a_terminal\";\
            identifier2 -> \"b_terminal\"\
                            | #;
            identifier3 -> \"c_terminal\"| #;
";
        let mut peekable = PeekableWrapper::<PeekableWrapper<Chars>>::new(to_parse.chars().peekable());
        let mut vm = NullVm::new();
        let mut rule_parser = RuleParser::new(&mut peekable, &mut vm);
        let _ = rule_parser.parse_rules().unwrap().rules;
        let first_dict = get_first_sets(&rule_parser.parser_data).unwrap();
        assert_eq!(make_memberset("b#"), first_dict.get(&rule_parser.parser_data.get_element_nt_index("identifier2").unwrap()).unwrap().clone());
        assert_eq!(make_memberset("c#"), first_dict.get(&rule_parser.parser_data.get_element_nt_index("identifier3").unwrap()).unwrap().clone());
        assert_eq!(make_memberset("abc#"), first_dict.get(&rule_parser.parser_data.get_element_nt_index("start").unwrap()).unwrap().clone());
    }

    #[test]
    fn test_first_list() {
        let to_parse =
            "start      -> list;\
            list -> l_element list_s ;\
            list_s -> l_element list_s| #;\
            l_element -> \"a\";\
";
        let mut peekable = PeekableWrapper::<PeekableWrapper<Chars>>::new(to_parse.chars().peekable());
        let mut vm = NullVm::new();
        let mut rule_parser = RuleParser::new(&mut peekable, &mut vm);
        let _ = rule_parser.parse_rules().unwrap().rules;
        let first_dict = get_first_sets(&rule_parser.parser_data).unwrap();
        assert_eq!(make_memberset("a"), first_dict.get(&rule_parser.parser_data.get_element_nt_index("start").unwrap()).unwrap().clone());
        assert_eq!(make_memberset("a"), first_dict.get(&rule_parser.parser_data.get_element_nt_index("list").unwrap()).unwrap().clone());
        assert_eq!(make_memberset("a#"), first_dict.get(&rule_parser.parser_data.get_element_nt_index("list_s").unwrap()).unwrap().clone());
        assert_eq!(make_memberset("a"), first_dict.get(&rule_parser.parser_data.get_element_nt_index("l_element").unwrap()).unwrap().clone());
    }


    #[test]
    fn test_first_set_of_partial() {
        let elements = vec![0];
        let mut first_sets = NamedSets::new();
        let  parser_data = ParserData::<NullVm>::new();
        first_sets.insert(0, make_memberset("c#"));
        let partial = first_set_of_partial(&elements, &first_sets, &parser_data).unwrap();
        assert_eq!(make_memberset("c#"), partial);
    }

    #[test]
    fn test_first_set_of_partial2() {
        let elements = vec![0,1];
        let mut first_sets = NamedSets::new();
        let parser_data = ParserData::<NullVm>::new();
        first_sets.insert(0, make_memberset("c#"));
        first_sets.insert(1, make_memberset("d#"));
        let partial = first_set_of_partial(&elements, &first_sets, &parser_data).unwrap();
        assert_eq!(make_memberset("cd#"), partial);
    }

    #[test]
    fn test_first_set_of_partial3() {
        let elements = vec![0,1,2];
        let mut first_sets = NamedSets::new();
        let parser_data = ParserData::<NullVm>::new();
        first_sets.insert(0, make_memberset("c#"));
        first_sets.insert(1, make_memberset("d"));
        first_sets.insert(2, make_memberset("e#"));
        let partial = first_set_of_partial(&elements, &first_sets, &parser_data).unwrap();
        assert_eq!(make_memberset("cd"), partial);
    }


    #[test]
    fn test_first_mengen_hard() {
        let to_parse =
            "start      -> identifier2 list identifier3;\
            identifier2 -> \"b_terminal\"\
                            | #;
            identifier3 -> \"c_terminal\"| #;
            list -> liststart listt;
            liststart-> \"list\"| #;
            listt -> # | listelement listt;
            listelement -> \"g\" | \"h\";
";
        let mut peekable = PeekableWrapper::<PeekableWrapper<Chars>>::new(to_parse.chars().peekable());
        let mut vm = NullVm::new();
        let mut rule_parser = RuleParser::new(&mut peekable, &mut vm);
        rule_parser.parse_rules().expect("TODO: panic message");
        let parser_data=rule_parser.parser_data;
        let first_dict = get_first_sets(&parser_data).unwrap();
        assert_eq!(make_memberset("blcx"), first_dict.get(&parser_data.get_element_nt_index("start").unwrap()).unwrap().clone());
        assert_eq!(make_memberset("c#"), first_dict.get(&parser_data.get_element_nt_index("identifier3").unwrap()).unwrap().clone());
        assert_eq!(make_memberset("lghx"), first_dict.get(&parser_data.get_element_nt_index("list").unwrap()).unwrap().clone());
        assert_eq!(make_memberset("#gh"), first_dict.get(&parser_data.get_element_nt_index("listt").unwrap()).unwrap().clone());
        assert_eq!(make_memberset("gh"), first_dict.get(&parser_data.get_element_nt_index("listelement").unwrap()).unwrap().clone());
        assert_eq!(make_memberset("xl"), first_dict.get(&parser_data.get_element_nt_index("liststart").unwrap()).unwrap().clone());
        assert_eq!(make_memberset("b#"), first_dict.get(&parser_data.get_element_nt_index("identifier2").unwrap()).unwrap().clone());
    }
}