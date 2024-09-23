use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;
use std::iter::FromIterator;

use errors::GrammarError;
use parser_data::{ElementType, ParserData};
use vms::VM;

use crate::first_sets::first_set_of_partial;
use crate::named_graph::GraphNamedNodes;
use crate::parser_data::{ElementIndex, Production, RuleMap};
use crate::sets::{NamedSets, NamedSetsNoEmpty, SetMember, SetMemberWithEmpty};
use crate::simple_graph::NodeData;

pub type Graph = GraphNamedNodes<HashSet<SetMember>>;


pub fn get_follow_sets<T>(start: ElementIndex, first_sets: &NamedSets, parser_data: &ParserData<T>) -> Result<NamedSetsNoEmpty, GrammarError> where T: VM {
    let mut follow_graph = make_graph_with_index(&parser_data.parse_rules.rules)?;
    follow_graph.get_node_mut(start)?.data.insert(SetMember::Terminate);
    for (&el_index, nt_rules) in &parser_data.parse_rules.rules {
        for production in nt_rules.possible_productions.iter() {
            let prodi = &**production;
            if let Production::NotEmpty(prod) = prodi {
                graph_marking_for_rightside_elements(&prod, &mut follow_graph, &first_sets, el_index,&parser_data)?
            }
        }
    }
    make_follow_sets_from_marked_graph(&mut follow_graph)
}

fn make_graph_with_index<T>(rules: &RuleMap<T>) -> Result<Graph, GrammarError> where T: VM {
    let mut follow_graph = Graph::new();
    for (name, _) in rules.iter() {
        follow_graph.add_node(*name, HashSet::new())?;
    }
    Ok(follow_graph)
}

fn make_follow_sets_from_marked_graph(follow_graph: &mut Graph) -> Result<NamedSetsNoEmpty, GrammarError> {
    let mut follow_sets: NamedSetsNoEmpty = HashMap::new();
    let mut changes = true;
    while changes {
        changes = false;
        let mut successor_indexes = vec![];
        for (name, node_index) in follow_graph.names.iter() {
            let node: &NodeData<HashSet<SetMember>> = follow_graph.get_node_by_index(*node_index)?;
            let successors = follow_graph.successors(*name).unwrap();
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

fn graph_marking_for_rightside_elements<T>(prod: &Vec<ElementIndex>, follow_graph: &mut Graph, first_sets: &NamedSets, left_side: ElementIndex, parser_data: &ParserData<T>)
                                           -> Result<(), GrammarError> where T: VM {
    let der_len = prod.len();
    for (i, &el_index) in prod.iter().enumerate() {
        let element=parser_data.get_element(el_index).ok_or(GrammarError::MissingElementForIndex { index:  el_index})?.et;
        if let ElementType::NonTerminal = &element {
            let is_last_element = i >= der_len - 1;
            let mut add_edge = false;
            if is_last_element {
                add_edge = true;
            } else {
                let mut first_set_following = first_set_of_partial(&prod[i + 1..], first_sets, &parser_data)?;
                let has_empty = first_set_following.contains(&SetMemberWithEmpty::Empty);
                if has_empty {
                    first_set_following.remove(&SetMemberWithEmpty::Empty);
                    add_edge = true;
                }
                follow_graph.get_node_mut(el_index)?.data.extend(HashSet::<_>::from_iter(first_set_following.iter().map(|x| SetMember::try_from(*x).unwrap())));
            }
            if add_edge {
                follow_graph.add_edge(left_side, el_index)?;
            }
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
        let mut peekable = PeekableWrapper::<PeekableWrapper<Chars>>::new(to_parse.chars().peekable());
        let mut vm = NullVm::new();
        let mut rule_parser = RuleParser::new(&mut peekable, &mut vm);
        rule_parser.parse_rules().expect("TODO: panic message");
        let parser_data=rule_parser.parser_data;
        let first_dict = get_first_sets(&parser_data).unwrap();
        let follow_dict = get_follow_sets(parser_data.get_element_nt_index("start").unwrap(), &first_dict, &parser_data).unwrap();
        assert_eq!(make_memberset_no_empty("!"), follow_dict.get(&parser_data.get_element_nt_index("start").unwrap()).unwrap().clone());
        assert_eq!(make_memberset_no_empty("c!"), follow_dict.get(&parser_data.get_element_nt_index("identifier2").unwrap()).unwrap().clone());
        assert_eq!(make_memberset_no_empty("!"), follow_dict.get(&parser_data.get_element_nt_index("identifier3").unwrap()).unwrap().clone());
    }

    #[test]
    fn test_follow_mengen2() {
        let to_parse =
            "start      -> identifier2 identifier3 \"a_terminal\";\
            identifier2 -> \"b_terminal\" | #;
            identifier3 -> \"c_terminal\"| #;
";
        let mut peekable = PeekableWrapper::<PeekableWrapper<Chars>>::new(to_parse.chars().peekable());
        let mut vm = NullVm::new();
        let mut rule_parser = RuleParser::new(&mut peekable, &mut vm);
        rule_parser.parse_rules().expect("TODO: panic message");
        let parser_data=rule_parser.parser_data;
        let first_dict = get_first_sets(&parser_data).unwrap();
        let follow_dict = get_follow_sets(parser_data.get_element_nt_index("start").unwrap(),  &first_dict,&parser_data).unwrap();
        assert_eq!(make_memberset_no_empty("!"), follow_dict.get(&parser_data.get_element_nt_index("start").unwrap()).unwrap().clone());
        assert_eq!(make_memberset_no_empty("ac"), follow_dict.get(&parser_data.get_element_nt_index("identifier2").unwrap()).unwrap().clone());
        assert_eq!(make_memberset_no_empty("a"), follow_dict.get(&parser_data.get_element_nt_index("identifier3").unwrap()).unwrap().clone());
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
        let mut peekable = PeekableWrapper::<PeekableWrapper<Chars>>::new(to_parse.chars().peekable());
        let mut vm = NullVm::new();
        let mut rule_parser = RuleParser::new(&mut peekable, &mut vm);
        rule_parser.parse_rules().expect("TODO: panic message");
        let parser_data=rule_parser.parser_data;
        let first_dict = get_first_sets(&parser_data).unwrap();
        let follow_dict = get_follow_sets(parser_data.get_element_nt_index("start").unwrap(),  &first_dict,&parser_data).unwrap();
        assert_eq!(make_memberset_no_empty("!"), follow_dict.get(&parser_data.get_element_nt_index("start").unwrap()).unwrap().clone());
        assert_eq!(make_memberset_no_empty("!"), follow_dict.get(&parser_data.get_element_nt_index("identifier3").unwrap()).unwrap().clone());
        assert_eq!(make_memberset_no_empty("c!"), follow_dict.get(&parser_data.get_element_nt_index("list").unwrap()).unwrap().clone());
        assert_eq!(make_memberset_no_empty("c!"), follow_dict.get(&parser_data.get_element_nt_index("listt").unwrap()).unwrap().clone());
        assert_eq!(make_memberset_no_empty("c!gh"), follow_dict.get(&parser_data.get_element_nt_index("listelement").unwrap()).unwrap().clone());
        assert_eq!(make_memberset_no_empty("c!gh"), follow_dict.get(&parser_data.get_element_nt_index("liststart").unwrap()).unwrap().clone());
        assert_eq!(make_memberset_no_empty("lc!gh"), follow_dict.get(&parser_data.get_element_nt_index("identifier2").unwrap()).unwrap().clone());
    }

    #[test]
    fn test_follow_list() {
        let to_parse =
            "start      -> list;\
            list -> l_element list_s ;\
            list_s -> l_element list_s| #;\
            l_element -> \"a\";\
";
        let mut peekable = PeekableWrapper::<PeekableWrapper<Chars>>::new(to_parse.chars().peekable());
        let mut vm = NullVm::new();
        let mut rule_parser = RuleParser::new(&mut peekable, &mut vm);
        rule_parser.parse_rules().expect("TODO: panic message");
        let parser_data=rule_parser.parser_data;
        let first_dict = get_first_sets(&parser_data).unwrap();
        let follow_dict = get_follow_sets(parser_data.get_element_nt_index("start").unwrap(),  &first_dict,&parser_data).unwrap();
        assert_eq!(make_memberset_no_empty("!"), follow_dict.get(&parser_data.get_element_nt_index("start").unwrap()).unwrap().clone());
        assert_eq!(make_memberset_no_empty("!"), follow_dict.get(&parser_data.get_element_nt_index("list").unwrap()).unwrap().clone());
        assert_eq!(make_memberset_no_empty("!"), follow_dict.get(&parser_data.get_element_nt_index("list_s").unwrap()).unwrap().clone());
        assert_eq!(make_memberset_no_empty("a!"), follow_dict.get(&parser_data.get_element_nt_index("l_element").unwrap()).unwrap().clone());
    }
}