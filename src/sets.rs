use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;
use std::fmt;
use parser_data::{Element, ElementIndex};


#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub enum SetMemberWithEmpty {
    Char(char),
    Empty,
    Terminate,
}

impl fmt::Display for SetMemberWithEmpty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SetMemberWithEmpty::Char(x) => {write!(f, "'{}'", x)}
            SetMemberWithEmpty::Empty => {write!(f, "empty")}
            SetMemberWithEmpty::Terminate => {write!(f, "terminate")}
        }

    }
}

impl Into<String> for SetMemberWithEmpty{
    fn into(self) -> String {
        match self {
            SetMemberWithEmpty::Char(x) => {String::from(x)}
            SetMemberWithEmpty::Empty => {String::from("empty")}
            SetMemberWithEmpty::Terminate => {String::from("terminate")}
        }
    }
}

#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub enum SetMember {
    Char(char),
    Terminate,
}
impl fmt::Display for SetMember {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SetMember::Char(x) => {write!(f, "'{}'", x)}
            SetMember::Terminate => {write!(f, "terminate")}
        }

    }
}

impl Into<String> for SetMember{
    fn into(self) -> String {
        match self {
            SetMember::Char(x) => {String::from(x)}
            SetMember::Terminate => {String::from("terminate")}
        }
    }
}


pub type HashMapOfSets<Ta, Tb> = HashMap<Ta, HashSet<Tb>>;
pub type NamedSets = HashMapOfSets<ElementIndex, SetMemberWithEmpty>;
pub type NamedSetsNoEmpty = HashMapOfSets<ElementIndex, SetMember>;


impl TryFrom<SetMemberWithEmpty> for SetMember {
    type Error = &'static str;

    fn try_from(value: SetMemberWithEmpty) -> Result<Self, Self::Error> {
        match value {
            SetMemberWithEmpty::Char(x) => { Ok(SetMember::Char(x)) }
            SetMemberWithEmpty::Empty => { Err("was empty") }
            SetMemberWithEmpty::Terminate => { Ok(SetMember::Terminate) }
        }
    }
}

impl From<Option<&char>> for SetMember {
    fn from(value: Option<&char>) -> Self {
        match value {
            None => { SetMember::Terminate }
            Some(x) => { SetMember::Char(*x) }
        }
    }
}