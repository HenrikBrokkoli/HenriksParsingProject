use std::collections::{HashMap, HashSet};
use std::str::Chars;
use peekables::{PeekableWrapper};
use crate::errors::{GrammarError};
use crate::rule_parsing::{Element, ET,  Production, RuleMap};
use crate::sets::{NamedSets, SetMemberWithEmpty};
use crate::vms::{VM, NullVm};

///Computes the first set of a slice of elements. Usually used to get the first set of a partial right side of a production.
/// # Arguments
/// * `elements` a reference to a slice of elements. The first_set is computed on the basis that the elements are following in this order
/// * `first_sets` a named Set of first_sets. For every element in elements there has to be an entry in first_sets
///
/// For example, we have the production a -> element1 element2 element3 and want to start computing the follow_set of element1
/// For that we need the first_set of [element2, element3].
/// Our first_sets are {element2: ['a',empty], element3: ['b','c']}
/// Our return  will be {'a','b','c'}
pub fn first_set_of_partial(elements: &[Element], first_sets: &NamedSets) -> HashSet<SetMemberWithEmpty> {
    let mut set = HashSet::new();

    for element in elements {
        let cur_set = get_first_set_of_element(&element.el_type, first_sets).unwrap();
        let has_empty = cur_set.contains(&SetMemberWithEmpty::Empty);
        set.extend(cur_set);
        if !has_empty {
            set.remove(&SetMemberWithEmpty::Empty);
            break;
        }
    }
    set
}


pub fn get_first_sets<T>(rules: &RuleMap<T>) -> Result<NamedSets, GrammarError> where T: VM {
    let mut first_sets: NamedSets = HashMap::new();
    for (rule_name, _) in rules {
        get_set_first_set_of_element(&ET::NonTerminal(rule_name.clone()), rules, &mut first_sets)?;
    }
    Ok(first_sets)
}


fn get_set_first_set_of_element<T>(element: &ET, rules: &RuleMap<T>, first_sets: &mut NamedSets) -> Result<HashSet<SetMemberWithEmpty>, GrammarError> where T: VM {
    let set_maybe = get_first_set_of_element(element, first_sets);
    match set_maybe {
        None => {
            if let ET::NonTerminal(el) = element {
                let mut set: HashSet<SetMemberWithEmpty> = HashSet::new();

                let productions = match rules.get(el) {
                    None => { return Err(GrammarError::MissingProduction { name: String::from(el) }); }
                    Some(x) => x
                };

                for production in productions.possible_productions.iter() {
                    set.extend(&get_first_set_of_production(production, rules, first_sets)?)
                }
                first_sets.insert(el.to_string(), set.clone());
                Ok(set)
            } else {
                panic!("can not happen")
            }
        }
        Some(x) => Ok(x)
    }
}


fn get_first_set_of_element(element: &ET, first_sets: &NamedSets) -> Option<HashSet<SetMemberWithEmpty>> {
    match element {
        ET::Terminal(x) => {
            let maybe_char = x.chars().next();
            Some(match maybe_char {
                None => { HashSet::from([SetMemberWithEmpty::Empty]) }
                Some(ch) => HashSet::from([SetMemberWithEmpty::Char(ch)])
            })
        }
        ET::NonTerminal(x) => {
            first_sets.get(x).map(|entry| (*entry).clone())
        }
    }
}


fn get_first_set_of_production<T>(production: &Production, rules: &RuleMap<T>, first_sets: &mut NamedSets) -> Result<HashSet<SetMemberWithEmpty>, GrammarError> where T:VM {
    match production {
        Production::Empty => Ok(HashSet::from([SetMemberWithEmpty::Empty])),
        Production::NotEmpty(elements) => {
            let mut first_menge = get_set_first_set_of_element(&elements.first().unwrap().el_type, rules, first_sets)?;
            if first_menge.contains(&SetMemberWithEmpty::Empty) {
                for element in elements.iter().skip(1) {
                    let cur_set = get_set_first_set_of_element(&element.el_type, rules, first_sets)?;
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
        let mut peekable= PeekableWrapper::<PeekableWrapper<Chars>>::new(to_parse.chars().peekable());
        let mut vm=NullVm::new();
        let mut rule_parser = RuleParser::new(&mut peekable, &mut vm);
        let rules = rule_parser.parse_rules().unwrap();
        let first_dict = get_first_sets(&rules.rules).unwrap();
        assert_eq!(make_memberset("abc#"), first_dict.get("start").unwrap().clone());
        assert_eq!(make_memberset("b#"), first_dict.get("identifier2").unwrap().clone());
        assert_eq!(make_memberset("c"), first_dict.get("identifier3").unwrap().clone())
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
        let mut peekable= PeekableWrapper::<PeekableWrapper<Chars>>::new(to_parse.chars().peekable());
        let mut vm=NullVm::new();
        let mut rule_parser = RuleParser::new(&mut peekable, &mut vm);
        let rules = rule_parser.parse_rules().unwrap();
        let first_dict = get_first_sets(&rules.rules).unwrap();
        assert_eq!(make_memberset("b#"), first_dict.get("identifier2").unwrap().clone());
        assert_eq!(make_memberset("c"), first_dict.get("identifier3").unwrap().clone());
        assert_eq!(make_memberset("abc"), first_dict.get("start").unwrap().clone());
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
        let mut peekable= PeekableWrapper::<PeekableWrapper<Chars>>::new(to_parse.chars().peekable());
        let mut vm=NullVm::new();
        let mut rule_parser = RuleParser::new(&mut peekable, &mut vm);
        let rules = rule_parser.parse_rules().unwrap();
        let first_dict = get_first_sets(&rules.rules).unwrap();
        assert_eq!(make_memberset("b#"), first_dict.get("identifier2").unwrap().clone());
        assert_eq!(make_memberset("c#"), first_dict.get("identifier3").unwrap().clone());
        assert_eq!(make_memberset("abc#"), first_dict.get("start").unwrap().clone());
    }

    #[test]
    fn test_first_list() {
        let to_parse =
            "start      -> list;\
            list -> l_element list_s ;\
            list_s -> l_element list_s| #;\
            l_element -> \"a\";\
";
        let mut peekable= PeekableWrapper::<PeekableWrapper<Chars>>::new(to_parse.chars().peekable());
        let mut vm=NullVm::new();
        let mut rule_parser = RuleParser::new(&mut peekable, &mut vm);
        let rules = rule_parser.parse_rules().unwrap();
        let first_dict = get_first_sets(&rules.rules).unwrap();
        assert_eq!(make_memberset("a"), first_dict.get("start").unwrap().clone());
        assert_eq!(make_memberset("a"), first_dict.get("list").unwrap().clone());
        assert_eq!(make_memberset("a#"), first_dict.get("list_s").unwrap().clone());
        assert_eq!(make_memberset("a"), first_dict.get("l_element").unwrap().clone());
    }


    #[test]
    fn test_first_set_of_partial() {
        let elements = vec![Element::new(ET::NonTerminal("identifier1".to_string()), false)];
        let mut first_sets = NamedSets::new();

        first_sets.insert("identifier1".to_string(), make_memberset("c#"));
        let partial = first_set_of_partial(&elements, &first_sets);
        assert_eq!(make_memberset("c#"), partial);
    }

    #[test]
    fn test_first_set_of_partial2() {
        let elements = vec![Element::new(ET::NonTerminal("identifier1".to_string()), false), Element::new(ET::NonTerminal("identifier2".to_string()), false)];
        let mut first_sets = NamedSets::new();
        first_sets.insert("identifier1".to_string(), make_memberset("c#"));
        first_sets.insert("identifier2".to_string(), make_memberset("d#"));
        let partial = first_set_of_partial(&elements, &first_sets);
        assert_eq!(make_memberset("cd#"), partial);
    }

    #[test]
    fn test_first_set_of_partial3() {
        let elements = vec![
            Element::new(ET::NonTerminal("identifier1".to_string()), false),
            Element::new(ET::NonTerminal("identifier2".to_string()), false),
            Element::new(ET::NonTerminal("identifier3".to_string()), false)];
        let mut first_sets = NamedSets::new();
        first_sets.insert("identifier1".to_string(), make_memberset("c#"));
        first_sets.insert("identifier2".to_string(), make_memberset("d"));
        first_sets.insert("identifier3".to_string(), make_memberset("e#"));
        let partial = first_set_of_partial(&elements, &first_sets);
        assert_eq!(make_memberset("cd"), partial);
    }
}