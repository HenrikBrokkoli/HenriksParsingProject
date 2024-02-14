use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;
use std::iter::FromIterator;
use vms::VM;
use crate::first_sets::first_set_of_partial;
use crate::named_graph::{GraphError, GraphNamedNodes};
use crate::rule_parsing::{Element, ET, Production, RuleMap};
use crate::sets::{NamedSets, NamedSetsNoEmpty, SetMemberWithEmpty, SetMember};
use crate::simple_graph::NodeData;

pub fn get_follow_sets<T>(start: String, rules: &RuleMap<T>, first_sets: &NamedSets)  -> Result<NamedSetsNoEmpty, GraphError> where T:VM {
    let mut follow_graph = make_graph_with_index(rules)?;
    follow_graph.get_node_mut(start.as_str())?.data.insert(SetMember::Terminate);
    for (name, nt_rules) in rules {
        for production in nt_rules.possible_productions.iter() {
            let prodi = &**production;
            if let Production::NotEmpty(prod) = prodi {
                graph_marking_for_rightside_elements(&prod, &mut follow_graph, &first_sets, name.clone())?
            }
        }
    }
    make_follow_sets_from_marked_graph(&mut follow_graph)
}

fn make_graph_with_index<T>(rules: &RuleMap<T>) -> Result<GraphNamedNodes<HashSet<SetMember>>, GraphError> where T:VM {
    let mut follow_graph = GraphNamedNodes::<HashSet<SetMember>>::new();
    for (name, _) in rules.iter() {
        follow_graph.add_node(String::from(name), HashSet::new())?;
    }
    Ok(follow_graph)
}

fn make_follow_sets_from_marked_graph(follow_graph: &mut GraphNamedNodes<HashSet<SetMember>>) -> Result<NamedSetsNoEmpty, GraphError> {
    let mut follow_sets: NamedSetsNoEmpty = HashMap::new();
    let mut changes = true;
    while changes {
        changes = false;
        let mut successor_indexes = vec![];
        for (name, node_index) in follow_graph.names.iter() {
            let node: &NodeData<HashSet<SetMember>> = follow_graph.get_node_by_index(*node_index)?;
            let successors = follow_graph.successors(name).unwrap();
            for (successor_index, successor) in successors {
                if !node.data.is_subset(&successor.data) {
                    changes = true;
                    let missing_in_successor = node.data.difference(&successor.data).map(|&x| x.clone()).collect::<Vec<SetMember>>();
                    successor_indexes.push((successor_index, missing_in_successor));
                }
            }
        }
        for (index, members_missing) in successor_indexes.iter() {
            follow_graph.get_node_mut_by_index(*index)?.data.extend(members_missing);
        }
    }
    for (name, index) in follow_graph.names.iter() {
        follow_sets.insert(name.clone(), follow_graph.get_node_by_index(*index)?.data.clone());
    }
    Ok(follow_sets)
}

fn graph_marking_for_rightside_elements(prod: &Vec<Element>, follow_graph: &mut GraphNamedNodes<HashSet<SetMember>>, first_sets: &NamedSets, left_side_name: String) -> Result<(), GraphError> {
    let der_len = prod.len();
    for (i, element) in prod.iter().enumerate() {
        if let ET::NonTerminal(name) = &element.el_type {
            let is_last_element = i >= der_len - 1;
            let mut add_edge = false;
            if is_last_element {
                add_edge = true;
            } else {
                let mut first_set_following = first_set_of_partial(&prod[i + 1..], first_sets);
                let has_empty = first_set_following.contains(&SetMemberWithEmpty::Empty);
                if has_empty {
                    first_set_following.remove(&SetMemberWithEmpty::Empty);
                    add_edge = true;
                }
                follow_graph.get_node_mut(name.as_str())?.data.extend(HashSet::<_>::from_iter(first_set_following.iter().map(|x| SetMember::try_from(*x).unwrap())));
            }
            if add_edge {
                follow_graph.add_edge(left_side_name.as_str(), name.as_str())?;
            }
        }
    }
    Ok(())
}



#[cfg(test)]
mod tests {
    use std::str::Chars;
    use peekables::PeekableWrapper;
    use test_helpers::make_memberset;
    use vms::NullVm;
    use crate::first_sets::get_first_sets;
    use crate::rule_parsing::RuleParser;
    use crate::test_helpers::make_memberset_no_empty;
    use super::*;


    #[test]
    fn test_follow_mengen() {
        let to_parse =
            "start      -> identifier2 identifier3
                |\"a_terminal\";\
identifier2 -> \"b_terminal\"\
               | #;
identifier3 -> \"c_terminal\"| #;
";
        let mut peekable= PeekableWrapper::<PeekableWrapper<Chars>>::new(to_parse.chars().peekable());
        let mut vm=NullVm::new();
        let mut rule_parser = RuleParser::new(&mut peekable, &mut vm);
        let rules = &rule_parser.parse_rules().unwrap().rules;
        let first_dict = get_first_sets( &rules).unwrap();
        let follow_dict = get_follow_sets("start".to_string(), &rules, &first_dict).unwrap();
        assert_eq!(make_memberset_no_empty("!"), follow_dict.get("start").unwrap().clone());
        assert_eq!(make_memberset_no_empty("c!"), follow_dict.get("identifier2").unwrap().clone());
        assert_eq!(make_memberset_no_empty("!"), follow_dict.get("identifier3").unwrap().clone());
    }

    #[test]
    fn test_follow_mengen2() {
        let to_parse =
            "start      -> identifier2 identifier3 \"a_terminal\";\
            identifier2 -> \"b_terminal\" | #;
            identifier3 -> \"c_terminal\"| #;
";
        let mut peekable= PeekableWrapper::<PeekableWrapper<Chars>>::new(to_parse.chars().peekable());
        let mut vm=NullVm::new();
        let mut rule_parser = RuleParser::new(&mut peekable, &mut vm);
        let rules = &rule_parser.parse_rules().unwrap().rules;
        let first_dict = get_first_sets(&rules).unwrap();
        let follow_dict = get_follow_sets("start".to_string(), &rules, &first_dict).unwrap();
        assert_eq!(make_memberset_no_empty("!"), follow_dict.get("start").unwrap().clone());
        assert_eq!(make_memberset_no_empty("ac"), follow_dict.get("identifier2").unwrap().clone());
        assert_eq!(make_memberset_no_empty("a"), follow_dict.get("identifier3").unwrap().clone());
    }

    #[test]
    fn test_follow_mengen3() {
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
        let mut peekable= PeekableWrapper::<PeekableWrapper<Chars>>::new(to_parse.chars().peekable());
        let mut vm=NullVm::new();
        let mut rule_parser = RuleParser::new(&mut peekable, &mut vm);
        let rules = &rule_parser.parse_rules().unwrap().rules;
        let first_dict = get_first_sets( &rules).unwrap();
        let follow_dict = get_follow_sets("start".to_string(), &rules, &first_dict).unwrap();
        assert_eq!(make_memberset_no_empty("!"), follow_dict.get("start").unwrap().clone());
        assert_eq!(make_memberset_no_empty("!"), follow_dict.get("identifier3").unwrap().clone());
        assert_eq!(make_memberset_no_empty("c!"), follow_dict.get("list").unwrap().clone());
        assert_eq!(make_memberset_no_empty("c!"), follow_dict.get("listt").unwrap().clone());
        assert_eq!(make_memberset_no_empty("c!gh"), follow_dict.get("listelement").unwrap().clone());
        assert_eq!(make_memberset_no_empty("c!gh"), follow_dict.get("liststart").unwrap().clone());
        assert_eq!(make_memberset_no_empty("lc!gh"), follow_dict.get("identifier2").unwrap().clone());
    }

    #[test]
    fn test_follow_list() {
        let to_parse =
            "start      -> list;\
            list -> l_element list_s ;\
            list_s -> l_element list_s| #;\
            l_element -> \"a\";\
";
        let mut peekable= PeekableWrapper::<PeekableWrapper<Chars>>::new(to_parse.chars().peekable());
        let mut vm=NullVm::new();
        let mut rule_parser = RuleParser::new(&mut peekable, &mut vm);
        let rules = &rule_parser.parse_rules().unwrap().rules;
        let first_dict = get_first_sets( &rules).unwrap();
        let follow_dict = get_follow_sets("start".to_string(), &rules, &first_dict).unwrap();
        assert_eq!(make_memberset_no_empty("!"), follow_dict.get("start").unwrap().clone());
        assert_eq!(make_memberset_no_empty("!"), follow_dict.get("list").unwrap().clone());
        assert_eq!(make_memberset_no_empty("!"), follow_dict.get("list_s").unwrap().clone());
        assert_eq!(make_memberset_no_empty("a!"), follow_dict.get("l_element").unwrap().clone());
    }
}