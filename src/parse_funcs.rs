//! Low-level helper functions used by the grammar and script parsers.
//!
//! These functions parse small lexical units (whitespace, digits, identifiers)
//! from a ParseProcess and are reusable building blocks for higher-level
//! parsers.

use crate::errors::ParserError;
use crate::errors::ParserError::{EndOfCharsError, Impossible, UnexpectedCharError};
use crate::peekables::{ParseProcess, TPeekable};

pub fn parse_whitespace<T>(to_parse: &mut ParseProcess<T>)
where
    T: TPeekable<Item = char>,
{
    while let Some(x) = to_parse.peek() {
        if !x.is_whitespace() {
            break;
        }
        to_parse.next();
    }
}

pub fn parse_digits<T>(to_parse: &mut ParseProcess<T>) -> Result<String, ParserError>
where
    T: TPeekable<Item = char>,
{
    let mut cur_char = to_parse.peek();
    let mut number_string = String::new();
    while let Some(x) = cur_char {
        if !x.is_ascii_digit() {
            break;
        }
        let pos = to_parse.cur_pos();
        number_string.push(to_parse.next().ok_or(EndOfCharsError { pos })?);
        cur_char = to_parse.peek();
    }
    Ok(number_string)
}

fn parse_required_digits<T>(to_parse: &mut ParseProcess<T>) -> Result<String, ParserError>
where
    T: TPeekable<Item = char>,
{
    let pos = to_parse.cur_pos();
    let cur_char = to_parse.peek().ok_or(EndOfCharsError { pos })?;
    if !cur_char.is_ascii_digit() {
        return Err(UnexpectedCharError {
            chr: *cur_char,
            pos: to_parse.cur_pos(),
            expected: String::from("digit expected"),
        });
    }

    parse_digits(to_parse)
}

pub fn parse_usize<T>(to_parse: &mut ParseProcess<T>) -> Result<usize, ParserError>
where
    T: TPeekable<Item = char>,
{
    parse_required_digits(to_parse)?
        .parse::<usize>()
        .map_err(|_| Impossible)
}

pub fn parse_isize<T>(to_parse: &mut ParseProcess<T>) -> Result<isize, ParserError>
where
    T: TPeekable<Item = char>,
{
    let pos = to_parse.cur_pos();
    let cur_char = *to_parse.peek().ok_or(EndOfCharsError { pos })?;

    let mut number_string = String::new();
    if cur_char == '-' {
        to_parse.next();
        number_string.push('-');
    } else if !cur_char.is_ascii_digit() {
        return Err(UnexpectedCharError {
            chr: cur_char,
            pos: to_parse.cur_pos(),
            expected: String::from("digit or - expected"),
        });
    }

    number_string.push_str(&parse_required_digits(to_parse)?);
    number_string.parse::<isize>().map_err(|_| Impossible)
}
pub fn parse_symbol<T>(to_parse: &mut ParseProcess<T>, sym: char) -> Result<(), ParserError>
where
    T: TPeekable<Item = char>,
{
    if let Some(chr) = to_parse.peek() {
        return if *chr == sym {
            to_parse.next();
            Ok(())
        } else {
            Err(UnexpectedCharError {
                chr: *chr,
                pos: to_parse.cur_pos(),
                expected: String::from(sym),
            })
        };
    }
    Err(EndOfCharsError {
        pos: to_parse.cur_pos(),
    })
}

pub fn parse_var_name<T>(to_parse: &mut ParseProcess<T>) -> Result<String, ParserError>
where
    T: TPeekable<Item = char>,
{
    let pos = to_parse.cur_pos();
    let cur_char = to_parse.peek().ok_or(EndOfCharsError { pos })?;
    if !cur_char.is_alphabetic() {
        return Err(UnexpectedCharError {
            chr: *cur_char,
            pos: to_parse.cur_pos(),
            expected: String::from("alphabetic character"),
        });
    }
    let mut id_name = cur_char.to_string();
    to_parse.next();
    let mut cur_char = to_parse.peek();
    while let Some(x) = cur_char {
        if !(x.is_alphanumeric() || *x == '_') {
            break;
        }
        id_name.push(to_parse.next().ok_or(EndOfCharsError {
            pos: to_parse.cur_pos(),
        })?);
        cur_char = to_parse.peek();
    }
    Ok(id_name)
}

#[cfg(test)]
mod tests {
    use crate::errors::ParserError;
    use crate::parse_funcs::{parse_isize, parse_usize};
    use crate::peekables::{ParseProcess, PeekableWrapper, TPeekable};

    #[test]
    fn test_parse_isize_negative() {
        let mut peekable = PeekableWrapper::from_str("-12");
        let mut parse_process = ParseProcess::new(&mut peekable, None, None);
        let number = parse_isize(&mut parse_process).unwrap();
        assert_eq!(number, -12);
    }

    #[test]
    fn test_parse_isize_positive() {
        let mut peekable = PeekableWrapper::from_str("3");
        let mut parse_process = ParseProcess::new(&mut peekable, None, None);
        let number = parse_isize(&mut parse_process).unwrap();
        assert_eq!(number, 3);
    }

    #[test]
    fn test_parse_isize_zero() {
        let mut peekable = PeekableWrapper::from_str("0");
        let mut parse_process = ParseProcess::new(&mut peekable, None, None);
        let number = parse_isize(&mut parse_process).unwrap();
        assert_eq!(number, 0);
    }

    #[test]
    fn test_parse_isize_stops_before_non_digit() {
        let mut peekable = PeekableWrapper::from_str("-12x");
        let mut parse_process = ParseProcess::new(&mut peekable, None, None);
        let number = parse_isize(&mut parse_process).unwrap();
        assert_eq!(number, -12);
        assert_eq!(parse_process.peek(), Some(&'x'));
    }

    #[test]
    fn test_parse_isize_rejects_non_digit_start() {
        let mut peekable = PeekableWrapper::from_str("+12");
        let mut parse_process = ParseProcess::new(&mut peekable, None, None);
        let error = parse_isize(&mut parse_process).unwrap_err();
        match error {
            ParserError::UnexpectedCharError { chr, pos, expected } => {
                assert_eq!(chr, '+');
                assert_eq!(pos, 0);
                assert_eq!(expected, "digit or - expected");
            }
            _ => panic!("unexpected error"),
        }
    }

    #[test]
    fn test_parse_isize_rejects_sign_without_digits() {
        let mut peekable = PeekableWrapper::from_str("-");
        let mut parse_process = ParseProcess::new(&mut peekable, None, None);
        let error = parse_isize(&mut parse_process).unwrap_err();
        match error {
            ParserError::EndOfCharsError { pos } => assert_eq!(pos, 1),
            _ => panic!("unexpected error"),
        }
    }

    #[test]
    fn test_parse_usize_positive() {
        let mut peekable = PeekableWrapper::from_str("123");
        let mut parse_process = ParseProcess::new(&mut peekable, None, None);
        let number = parse_usize(&mut parse_process).unwrap();
        assert_eq!(number, 123);
    }

    #[test]
    fn test_parse_usize_zero() {
        let mut peekable = PeekableWrapper::from_str("0");
        let mut parse_process = ParseProcess::new(&mut peekable, None, None);
        let number = parse_usize(&mut parse_process).unwrap();
        assert_eq!(number, 0);
    }

    #[test]
    fn test_parse_usize_stops_before_non_digit() {
        let mut peekable = PeekableWrapper::from_str("42a");
        let mut parse_process = ParseProcess::new(&mut peekable, None, None);
        let number = parse_usize(&mut parse_process).unwrap();
        assert_eq!(number, 42);
        assert_eq!(parse_process.peek(), Some(&'a'));
    }

    #[test]
    fn test_parse_usize_rejects_negative_sign() {
        let mut peekable = PeekableWrapper::from_str("-12");
        let mut parse_process = ParseProcess::new(&mut peekable, None, None);
        let error = parse_usize(&mut parse_process).unwrap_err();
        match error {
            ParserError::UnexpectedCharError { chr, pos, expected } => {
                assert_eq!(chr, '-');
                assert_eq!(pos, 0);
                assert_eq!(expected, "digit expected");
            }
            _ => panic!("unexpected error"),
        }
    }

    #[test]
    fn test_parse_usize_requires_digit() {
        let mut peekable = PeekableWrapper::from_str("");
        let mut parse_process = ParseProcess::new(&mut peekable, None, None);
        let error = parse_usize(&mut parse_process).unwrap_err();
        match error {
            ParserError::EndOfCharsError { pos } => assert_eq!(pos, 0),
            _ => panic!("unexpected error"),
        }
    }
}
