use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use vms::{Instruction, VM};
use crate::errors::GrammarError;
use crate::rule_parsing::{ ET, Production, RuleMap};

use crate::errors::GrammarError::{MissingFollowSet, MissingSteuerSet, SteuerSetsNotDistinct};
use crate::sets::{NamedSets, NamedSetsNoEmpty, SetMember};
use crate::steuer_sets::get_steuer_sets;

pub struct NTRules<T>{
    pub steuermap: Steuermap,
    pub ignore:Option<String>,
    pub instruction: Option<Box<Instruction<T>>>
}
pub type Steuermap = HashMap<SetMember, Rc<Production>>;

pub fn get_steuermaps<'vm,T>(rules: RuleMap<'vm,T>, first_sets: &NamedSets, follow_sets: &NamedSetsNoEmpty) -> Result<HashMap<String, NTRules<T::Tstate>>, GrammarError>  where T:VM{
    let steuer_sets = get_steuer_sets(first_sets, follow_sets)?;
    let mut steuer_maps = HashMap::new();
    for (rule_name, productions) in rules.into_iter() {
        let mut steuermap = Steuermap::new();

        for prod in productions.possible_productions.into_iter() {
            steuermap_of_production(&steuer_sets, follow_sets, &mut steuermap, prod, &rule_name.clone())?;
        }
        steuer_maps.insert(rule_name, NTRules{steuermap, ignore:productions.ignore.clone(),instruction:productions.instruction});
    }
    Ok(steuer_maps)
}

fn steuermap_of_production(steuer_sets: &NamedSetsNoEmpty, follow_sets: &NamedSetsNoEmpty, steuer_map: &mut Steuermap, prod: Rc<Production>, cur_rule_name: &str) -> Result<(), GrammarError> {
    let prod_ref = Rc::clone(&prod);
    let prod = &*prod;
    match prod {
        Production::NotEmpty(el) => {
            let first = &el[0].el_type;
            match first {
                ET::Terminal(ter) => {
                    let old_key = steuer_map.insert(SetMember::Char(ter.chars().next().unwrap()), prod_ref);
                    if old_key.is_some() {
                        return Err(SteuerSetsNotDistinct{
                            steuer_terminal: ter.clone(),
                            steuer_char:ter.chars().next().unwrap(),
                            rule_name: cur_rule_name.to_string(),
                        });
                    }
                }
                ET::NonTerminal(nonter) => {
                    let steuer_menge = steuer_sets.get(nonter).ok_or(MissingSteuerSet { name: nonter.clone() })?;
                    fill_with_steuer_set(steuer_menge, steuer_map, prod_ref,cur_rule_name)?;
                }
            }
            Ok(())
        }
        Production::Empty => {
            let follow_set = follow_sets.get(cur_rule_name).ok_or(MissingFollowSet { name: String::from(cur_rule_name) })?;
            fill_with_steuer_set(follow_set, steuer_map, prod_ref,cur_rule_name)?;

            Ok(())
        }
    }
}


fn fill_with_steuer_set(set_no_empty: &HashSet<SetMember>, steuer_map: &mut Steuermap, prod: Rc<Production>, cur_rule_name: &str) -> Result<(), GrammarError> {
    for follow_char in set_no_empty.iter()
    {
        let old_key = steuer_map.insert(*follow_char, Rc::clone(&prod));
        if old_key.is_some() {
            return Err(SteuerSetsNotDistinct{
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
    use std::rc::Rc;
    use std::str::Chars;
    use peekables::PeekableWrapper;
    use rule_parsing::RuleMap;
    use vms::NullVm;
    use crate::first_sets::get_first_sets;
    use crate::follow_sets::get_follow_sets;
    use crate::rule_parsing::RuleParser;
    use crate::sets::SetMember;
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
    fn test_1(){
        let to_parse=
            "start -> a|b;\
a -> \"astring\";\
b -> \"bstring\";\
";

        let mut peekable = PeekableWrapper::<PeekableWrapper<Chars>>::new(to_parse.chars().peekable());
        let mut vm = NullVm::new();
        let mut rule_parser = RuleParser::new(&mut peekable, &mut vm);
        let rules = &rule_parser.parse_rules().unwrap().rules;

        let first_dict = get_first_sets( &rules).unwrap();
        let follow_dict = get_follow_sets("start".to_string(), &rules, &first_dict).unwrap();
        let steuer_maps = get_steuermaps(rule_parser.parse_rules.rules, &first_dict, &follow_dict).unwrap();
    }

    #[test]
    fn test_two_with_same_first(){
        let to_parse=
            "start -> a a_a;\
a -> \"a\";\
a_a -> \"a\";\
";

        let mut peekable = PeekableWrapper::<PeekableWrapper<Chars>>::new(to_parse.chars().peekable());
        let mut vm = NullVm::new();
        let mut rule_parser = RuleParser::new(&mut peekable, &mut vm);
        let rules = &rule_parser.parse_rules().unwrap().rules;

        let first_dict = get_first_sets( &rules).unwrap();
        let follow_dict = get_follow_sets("start".to_string(), &rules, &first_dict).unwrap();
        let steuer_maps = get_steuermaps(rule_parser.parse_rules.rules, &first_dict, &follow_dict).unwrap();
    }

    #[test]
    fn test_steuer_map_list() {
        let rules =
            "start      -> list ;\
            list -> element list_s ;\
            list_s -> element list_s| #;\
            element -> \"a\";\
";

        let mut peekable= PeekableWrapper::<PeekableWrapper<Chars>>::new(rules.chars().peekable());
        let mut vm=NullVm::new();
        let mut rule_parser = RuleParser::new(&mut peekable, &mut vm);
        let rule_dict = &rule_parser.parse_rules().unwrap().rules;
        let first_dict = get_first_sets( &rule_dict).unwrap();
        let follow_dict = get_follow_sets("start".to_string(), &rule_dict,&first_dict).unwrap();

        let steuer_maps = get_steuermaps(rule_parser.parse_rules.rules, &first_dict, &follow_dict).unwrap();

    }






    #[test]
    fn test_more_whitespace_shenanigans(){
        let to_parse=
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
        let mut rule_parser = RuleParser::new(&mut peekable, &mut vm);
        let rules = &rule_parser.parse_rules().unwrap().rules;


        let first_dict = get_first_sets( rules).unwrap();
        let follow_dict = get_follow_sets("start".to_string(), rules, &first_dict).unwrap();
        let steuer_maps = get_steuermaps(rule_parser.parse_rules.rules, &first_dict, &follow_dict).unwrap();
    }
    #[test]
    fn test_more_whitespace_shenanigans3(){
        let to_parse=
            "$IGNORE: whitespaces;\
start  -> terms;\
\
terms -> term terms_s;\
terms_s -> term terms_s | #;\
\
\
term -> number;\

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
        let mut rule_parser = RuleParser::new(&mut peekable, &mut vm);
        let rules = &rule_parser.parse_rules().unwrap().rules;


        let first_dict = get_first_sets( rules).unwrap();
        let follow_dict = get_follow_sets("start".to_string(), rules, &first_dict).unwrap();
        let steuer_maps = get_steuermaps(rule_parser.parse_rules.rules, &first_dict, &follow_dict).unwrap();
    }
}