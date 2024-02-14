use std::collections::HashSet;
use std::convert::TryFrom;
use std::iter::FromIterator;
use crate::errors::GrammarError;
use crate::sets::{NamedSets, NamedSetsNoEmpty, SetMemberWithEmpty, SetMember};

pub fn get_steuer_sets(first_sets: &NamedSets, follow_sets: &NamedSetsNoEmpty) -> Result<NamedSetsNoEmpty, GrammarError> {
    let mut steuer_sets = NamedSetsNoEmpty::new();
    for (name, first) in first_sets.iter() {
        let mut steuer = HashSet::from_iter(first.iter().filter(|member| **member != SetMemberWithEmpty::Empty).map(|x| SetMember::try_from(*x).unwrap()));
        if first.contains(&SetMemberWithEmpty::Empty) {
            let follow = follow_sets.get(name).ok_or(GrammarError::MissingFollowSet { name: String::from(name) })?;
            steuer.extend(follow);
        }
        steuer_sets.insert(String::from(name), steuer);
    }
    Ok(steuer_sets)
}

#[cfg(test)]
mod tests {
    use std::str::Chars;
    use peekables::PeekableWrapper;
    use vms::NullVm;
    use crate::first_sets::get_first_sets;
    use crate::follow_sets::get_follow_sets;
    use crate::rule_parsing::RuleParser;
    use crate::steuer_sets::get_steuer_sets;
    use crate::test_helpers::make_memberset_no_empty;

    #[test]
    fn test_steuer_sets() {
        let mut rules =
            "start      -> identifier1
            identifier2;\
            identifier1 -> \"a_terminal\"| #;
            identifier2 -> \"b_terminal\"| #;
";

        let mut peekable = PeekableWrapper::<PeekableWrapper<Chars>>::new(rules.chars().peekable());
        let mut vm = NullVm::new();
        let mut parser = RuleParser::new(&mut peekable, &mut vm);
        let rule_dict = &parser.parse_rules().unwrap().rules;
        let first_dict = get_first_sets(rule_dict).unwrap();
        let follow_dict = get_follow_sets("start".to_string(), rule_dict, &first_dict).unwrap();
        let steuer_dict = get_steuer_sets(&first_dict, &follow_dict).unwrap();
        assert_eq!(make_memberset_no_empty("!ab"), steuer_dict.get("start").unwrap().clone());
        assert_eq!(make_memberset_no_empty("ab!"), steuer_dict.get("identifier1").unwrap().clone());
        assert_eq!(make_memberset_no_empty("b!"), steuer_dict.get("identifier2").unwrap().clone());
    }


    #[test]
    fn test_steuer_sets_list() {
        let rules =
            "start      -> list ;\
            list -> element list_s ;\
            list_s -> element list_s| #;\
            element -> \"a\";\
";

        let mut peekable = PeekableWrapper::<PeekableWrapper<Chars>>::new(rules.chars().peekable());
        let mut vm = NullVm::new();
        let mut parser = RuleParser::new(&mut peekable, &mut vm);
        let rule_dict = &parser.parse_rules().unwrap().rules;
        let first_dict = get_first_sets(&rule_dict).unwrap();
        let follow_dict = get_follow_sets("start".to_string(), &rule_dict, &first_dict).unwrap();
        let steuer_dict = get_steuer_sets(&first_dict, &follow_dict).unwrap();
        assert_eq!(make_memberset_no_empty("a"), steuer_dict.get("list").unwrap().clone());
        assert_eq!(make_memberset_no_empty("a!"), steuer_dict.get("list_s").unwrap().clone());
        assert_eq!(make_memberset_no_empty("a"), steuer_dict.get("element").unwrap().clone());
    }

    #[test]
    fn test_steuer_sets_optional_list() {
        let rules =
            "start      -> optional_list \"ende\";\
            optional_list-> list|#;
            list -> element list_s ;\
            list_s -> element list_s| #;\
            element -> \"a\";\
";

        let mut peekable = PeekableWrapper::<PeekableWrapper<Chars>>::new(rules.chars().peekable());
        let mut vm = NullVm::new();
        let mut parser = RuleParser::new(&mut peekable, &mut vm);
        let rule_dict = &parser.parse_rules().unwrap().rules;
        let first_dict = get_first_sets(&rule_dict).unwrap();
        let follow_dict = get_follow_sets("start".to_string(), &rule_dict, &first_dict).unwrap();
        let steuer_dict = get_steuer_sets(&first_dict, &follow_dict).unwrap();
        assert_eq!(make_memberset_no_empty("a"), steuer_dict.get("list").unwrap().clone());
        assert_eq!(make_memberset_no_empty("ae"), steuer_dict.get("list_s").unwrap().clone());
        assert_eq!(make_memberset_no_empty("a"), steuer_dict.get("element").unwrap().clone());

        assert_eq!(make_memberset_no_empty("ae"), steuer_dict.get("optional_list").unwrap().clone());
    }


    #[test]
    fn test_more_whitespace_shenanigans() {
        let to_parse =
            "$IGNORE: whitespaces;\
start -> terms;\
\
terms -> term terms_s;\
terms_s -> term terms_s | #;\
\
\
term -> add|sub|number|print;\
\
print -> \"print\";\
add -> \"+\";\
sub -> \"-\";\
\
number-> digit number_s;\
number_s -> number_s_ | #;\
number_s_ ->digit number_s;\
digit -> \"0\"|\"1\"|\"2\"|\"3\"|\"4\"|\"5\"|\"6\"|\"7\"|\"8\"|\"9\";\
\
whitespaces -> whitespace whitespaces_s;\
whitespaces_s -> whitespace whitespaces_s| #;\
whitespace -> \" \";";

        let mut peekable = PeekableWrapper::<PeekableWrapper<Chars>>::new(to_parse.chars().peekable());
        let mut vm = NullVm::new();
        let mut rule_parser = RuleParser::new(&mut peekable, &mut vm);
        let rules = &rule_parser.parse_rules().unwrap().rules;


        let first_dict = get_first_sets(rules).unwrap();
        let follow_dict = get_follow_sets("start".to_string(), rules, &first_dict).unwrap();
        let steuer_dict = get_steuer_sets(&first_dict, &follow_dict).unwrap();
        assert_eq!(make_memberset_no_empty("p+-0123456789"), steuer_dict.get("start").unwrap().clone());
        assert_eq!(make_memberset_no_empty("p+-0123456789! "), steuer_dict.get("terms_s").unwrap().clone());
        assert_eq!(make_memberset_no_empty("p+-0123456789"), steuer_dict.get("term").unwrap().clone());

        assert_eq!(make_memberset_no_empty("0123456789"), steuer_dict.get("number").unwrap().clone());
        assert_eq!(make_memberset_no_empty("0123456789p+- "), steuer_dict.get("number_s").unwrap().clone());
        assert_eq!(make_memberset_no_empty("0123456789"), steuer_dict.get("digit").unwrap().clone());
        assert_eq!(make_memberset_no_empty(" "), steuer_dict.get("whitespaces").unwrap().clone());
    }
}


