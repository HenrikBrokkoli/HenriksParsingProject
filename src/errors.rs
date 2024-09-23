use std::fmt;

use parser_data::ElementIndex;
use tree::TreeError;
use crate::errors::GrammarError::{MissingElementForIndex, MissingFirstSet, MissingFollowSet, MissingProduction, MissingSteuerSet};
use crate::errors::ParserError::{EndOfCharsError, UnexpectedCharError, UnknownSpecialOperation};

#[derive(Debug)]
pub enum ParserError {
    UnexpectedCharError { chr: char, pos: usize, expected: String },
    EndOfCharsError,
    GramError { err: GrammarError },
    UnknownSpecialOperation { operation: String, pos: usize },
    Impossible,
    InternalError { message: String },
    VmError { message: String },
    TreeError{err:TreeError}
}

impl std::error::Error for ParserError {}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            UnexpectedCharError { chr, pos, expected } => write!(f, " \"{}\" at pos {} was not expected. Expected {}", chr, pos, expected),
            EndOfCharsError => write!(f, "There was a char expected but there was none"),
            UnexpectedElementError => write!(f, "Unexpected Element"),
            UnknownSpecialOperation { operation, pos } => write!(f, " \"{}\" at pos {} was not expected", operation, pos),
        }
    }
}

impl From<GrammarError> for ParserError {
    fn from(err: GrammarError) -> Self {
        ParserError::GramError { err }
    }
}
impl From<TreeError> for ParserError{
    fn from(err: TreeError) -> Self {
        ParserError::TreeError { err }
    }
}

#[derive(Debug)]
pub enum GrammarError {
    MissingFollowSet { index: ElementIndex },
    MissingElementForIndex { index: ElementIndex },
    MissingFirstSet { index: ElementIndex },
    MissingSteuerSet { index: ElementIndex },
    MissingProduction { index: ElementIndex },
    SteuerSetsNotDistinct { steuer_terminal: String, steuer_char: char, rule_name: String },
    UnexpectedElementError { reason: String, pos: usize },
    GraphNodeAlreadyExistsError { node_name: usize },
    GraphNodeDoesNotExistsError { node_name: usize },
    GraphIndexOutOfBounds { index: usize },
}

impl std::error::Error for GrammarError {}

impl fmt::Display for GrammarError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MissingFollowSet { index } => write!(f, " \"{}\" has no followset", index),
            MissingElementForIndex { index } => write!(f, " \"{}\" has no entry in elements", index),
            MissingFirstSet { index } => write!(f, " \"{}\" has no firstset", index),
            MissingSteuerSet { index } => write!(f, " \"{}\" has no steuerset", index),
            MissingProduction { index } => write!(f, " \"{}\" not in productions", index),
            SteuerSetsNotDistinct => write!(f, "steuersets not distinct"),
            UnexpectedElementError => write!(f, "Unexpected Element"),
        }
    }
}