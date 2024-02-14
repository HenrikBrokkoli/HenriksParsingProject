use crate::errors::ParserError;
use crate::errors::ParserError::{EndOfCharsError, Impossible, UnexpectedCharError};
use crate::peekables::{ParseProcess, TPeekable};




pub fn parse_whitespace<T>(to_parse: &mut ParseProcess<T>) where T: TPeekable<Item=char> {
    while let Some(x) = to_parse.peek() {
        if !x.is_whitespace() {
            break;
        }
        to_parse.next();
    }
}

pub fn parse_digits<T>(to_parse: &mut ParseProcess<T>) -> Result<String, ParserError> where T: TPeekable<Item=char> {
    let mut cur_char = to_parse.peek();
    let mut number_string=String::new();
    while let Some(x) = cur_char {
        if !x.is_digit(10) {
            break;
        }
        number_string.push(to_parse.next().ok_or(EndOfCharsError)?);
        cur_char = to_parse.peek();
    };
    Ok(number_string)
}

pub fn parse_usize<T>(to_parse: &mut ParseProcess<T>) -> Result<usize, ParserError> where T: TPeekable<Item=char> {
    let cur_char = to_parse.peek().ok_or(EndOfCharsError)?;
    let mut numbers = cur_char.to_string();
    if !cur_char.is_digit(10) {
        return Err(UnexpectedCharError { chr: *cur_char, pos: to_parse.cur_pos() , expected:String::from("digit expected")});
    }
    to_parse.next();
    numbers.push_str(parse_digits(to_parse)?.as_str());
    let res = numbers.parse::<usize>();
    match res {
        Ok(x) => { Ok(x) }
        Err(_) => { Err(Impossible) }
    }
}

pub fn parse_isize<T>(to_parse: &mut ParseProcess<T>) -> Result<isize, ParserError> where T: TPeekable<Item=char> {
    let cur_char = *to_parse.peek().ok_or(EndOfCharsError)?;
    let mut numbers = cur_char.to_string();
    let has_sign = cur_char == '-';
    if has_sign {
        to_parse.next();
        numbers.push(cur_char);
    }

    let cur_char = to_parse.peek().ok_or(EndOfCharsError)?;
    if !cur_char.is_digit(10) {
        return Err(UnexpectedCharError { chr: *cur_char, pos: to_parse.cur_pos(),expected:String::from("digit or - expected") });
    }

    to_parse.next();
    numbers.push_str(parse_digits(to_parse)?.as_str());
    let res = numbers.parse::<isize>();
    match res {
        Ok(x) => { Ok(x) }
        Err(_) => { Err(Impossible) }
    }
}
pub fn parse_symbol<T>(to_parse: &mut ParseProcess<T>, sym: char) -> Result<(), ParserError> where T: TPeekable<Item=char> {
    if let Some(chr) = to_parse.peek() {
        return if *chr == sym {
            to_parse.next();
            Ok(())
        } else {
            Err(UnexpectedCharError { chr: *chr, pos: to_parse.cur_pos() ,expected:String::from(sym)})
        };
    }
    Err(EndOfCharsError)
}

pub fn parse_var_name<T>(to_parse: &mut ParseProcess<T>) -> Result<String, ParserError> where T: TPeekable<Item=char> {
    let cur_char = to_parse.peek().ok_or(EndOfCharsError)?;
    if !cur_char.is_alphabetic() {
        return Err(UnexpectedCharError { chr: *cur_char, pos: to_parse.cur_pos() , expected:String::from("alphabetic character")});
    }
    let mut id_name = cur_char.to_string();
    to_parse.next();
    let mut cur_char = to_parse.peek();
    while let Some(x) = cur_char {
        if !(x.is_alphanumeric()||*x=='_') {
            break;
        }
        id_name.push(to_parse.next().ok_or(EndOfCharsError)?);
        cur_char = to_parse.peek();
    }
    Ok(id_name)
}

