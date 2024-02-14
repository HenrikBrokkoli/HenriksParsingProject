use std::collections::HashSet;
use crate::sets::{SetMemberWithEmpty, SetMember};

pub fn make_memberset(chrs: &str) -> HashSet<SetMemberWithEmpty> {
    let mut set = HashSet::new();
    for x in chrs.chars() {
        if x == '#' {
            set.insert(SetMemberWithEmpty::Empty);
        } else if x == '!' { set.insert(SetMemberWithEmpty::Terminate); } else { set.insert(SetMemberWithEmpty::Char(x)); }
    }
    return set;
}

pub fn make_memberset_no_empty(chrs: &str) -> HashSet<SetMember> {
    let mut set = HashSet::new();
    for x in chrs.chars() {
        if x == '!' { set.insert(SetMember::Terminate); } else { set.insert(SetMember::Char(x)); }
    }
    return set;
}



#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use sets::{SetMember, SetMemberWithEmpty};
    use test_helpers::{make_memberset, make_memberset_no_empty};

    #[test]
    fn test_make_memberset() {
        let memberset= make_memberset("ab!#");
        assert_eq!(HashSet::<SetMemberWithEmpty>::from([SetMemberWithEmpty::Char('a'),SetMemberWithEmpty::Char('b'),SetMemberWithEmpty::Empty,SetMemberWithEmpty::Terminate]),memberset)

    }
    #[test]
    fn test_make_memberset_no_empty() {
        let memberset= make_memberset_no_empty("ab!#");
        assert_eq!(HashSet::<SetMember>::from([SetMember::Char('a'),SetMember::Char('b'),SetMember::Char('#'),SetMember::Terminate]),memberset)

    }
}