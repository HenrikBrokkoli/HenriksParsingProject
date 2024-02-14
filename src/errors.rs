use std::fmt;
use std::iter::FromIterator;
use crate::errors::GrammarError::{MissingFirstSet, MissingFollowSet, MissingProduction, MissingSteuerSet, };
use crate::errors::ParserError::{UnexpectedCharError, EndOfCharsError,UnknownSpecialOperation};

#[derive(Debug)]
pub enum ParserError {
    UnexpectedCharError { chr: char, pos: usize, expected:String },
    EndOfCharsError,
    GramError { err: GrammarError },
    UnknownSpecialOperation{operation:String,pos:usize},
    Impossible,
    InternalError{message:String},
    VmError {message:String}
}

impl std::error::Error for ParserError {}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            UnexpectedCharError { chr,pos ,expected} => write!(f, " \"{}\" at pos {} was not expected. Expected {}",chr,pos, expected),
            EndOfCharsError => write!(f, "There was a char expected but there was none"),
            UnexpectedElementError => write!(f, "Unexpected Element"),
            UnknownSpecialOperation{operation,pos} => write!(f, " \"{}\" at pos {} was not expected",operation,pos),
        }
    }
}

impl From<GrammarError> for ParserError {
    fn from(err: GrammarError) -> Self {
        ParserError::GramError { err }
    }
}

#[derive(Debug)]
pub enum GrammarError {
    MissingFollowSet { name: String },
    MissingFirstSet { name: String },
    MissingSteuerSet { name: String },
    MissingProduction { name: String },
    SteuerSetsNotDistinct {steuer_terminal:String, steuer_char:char, rule_name:String},
    UnexpectedElementError {reason: String, pos:usize},
}

impl std::error::Error for GrammarError {}

impl fmt::Display for GrammarError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MissingFollowSet { name } => write!(f, " \"{}\" has no followset", name),
            MissingFirstSet { name } => write!(f, " \"{}\" has no firstset", name),
            MissingSteuerSet { name } => write!(f, " \"{}\" has no steuerset", name),
            MissingProduction { name } => write!(f, " \"{}\" not in productions", name),
            SteuerSetsNotDistinct => write!(f, "steuersets not distinct"),
            UnexpectedElementError => write!(f, "Unexpected Element"),
        }
    }
}