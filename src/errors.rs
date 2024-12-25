use std::fmt;

use parser_data::ElementIndex;
use tree::TreeError;


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
            ParserError::UnexpectedCharError { chr, pos, expected } => write!(f, " \"{}\" at pos {} was not expected. Expected {}", chr, pos, expected),
            ParserError::EndOfCharsError => write!(f, "There was a char expected but there was none"),
            ParserError::UnknownSpecialOperation { operation, pos } => write!(f, " \"{}\" at pos {} was not expected", operation, pos),
            ParserError::GramError { .. } => write!(f, "There was a Grammar error"),
            ParserError::Impossible => write!(f,"This error should not be possible"),
            ParserError::InternalError { .. } => write!(f, "There was an internal error"),
            ParserError::VmError { .. } => write!(f, "There was a VM error"),
            ParserError::TreeError { .. } => write!(f, "There was a treeerror"),
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
            GrammarError::MissingFollowSet { index } => write!(f, " \"{}\" has no followset", index),
            GrammarError::MissingElementForIndex { index } => write!(f, " \"{}\" has no entry in elements", index),
            GrammarError::MissingFirstSet { index } => write!(f, " \"{}\" has no firstset", index),
            GrammarError::MissingSteuerSet { index } => write!(f, " \"{}\" has no steuerset", index),
            GrammarError::MissingProduction { index } => write!(f, " \"{}\" not in productions", index),
            GrammarError::SteuerSetsNotDistinct{ .. } => write!(f, "steuersets not distinct"),
            GrammarError::UnexpectedElementError { reason, pos } => write!(f, " \"{}\" at pos {} was not expected", reason, pos),
            GrammarError::GraphNodeAlreadyExistsError { .. } => write!(f, "graph node already exists"),
            GrammarError::GraphNodeDoesNotExistsError { .. } => write!(f, "graph node doesn't exists"),
            GrammarError::GraphIndexOutOfBounds { .. } => write!(f, "graph node index out of bounds"),
        }
    }
}