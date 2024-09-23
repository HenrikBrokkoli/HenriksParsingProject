use std::iter::Peekable;

pub trait TPeekable: Iterator {

    fn peek(&mut self) -> Option<&Self::Item>;

    fn peek_mut(&mut self) -> Option<&mut Self::Item>;

    fn next_if(
        &mut self,
        func: impl FnOnce(&Self::Item) -> bool) -> Option<Self::Item>;


    fn next_if_eq(&mut self, expected: &Self::Item) -> Option<Self::Item>;
}

pub struct PeekableWrapper<T> where T: Iterator, <T as Iterator>::Item: PartialEq {
    peekable: Peekable<T>,
}

impl<T> Iterator for PeekableWrapper<T>  where T: Iterator, <T as Iterator>::Item: PartialEq  {
    type Item = T::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.peekable.next()
    }
}

impl<T> TPeekable for PeekableWrapper<T>  where T: Iterator, <T as Iterator>::Item: PartialEq  {
    fn peek(&mut self) -> Option<&Self::Item> {
        self.peekable.peek()
    }

    fn peek_mut(&mut self) -> Option<&mut Self::Item> {
        self.peekable.peek_mut()
    }

    fn next_if(&mut self, func: impl FnOnce(&Self::Item) -> bool) -> Option<Self::Item> {
        self.peekable.next_if(func)
    }

    fn next_if_eq(&mut self, expected: &Self::Item) -> Option<Self::Item> {
        self.peekable.next_if_eq(expected)
    }
}

impl<T> PeekableWrapper<T> where T: Iterator, <T as Iterator>::Item: PartialEq{
    pub fn new<P>(peekable:Peekable<P>)->PeekableWrapper<P> where P: Iterator, <P as Iterator>::Item: PartialEq{
        PeekableWrapper{peekable}
    }
}


pub struct ParseProcess<'a, T> where T: TPeekable<Item=char> {
    to_parse: &'a mut T,
    current_position: usize,
    stop_on: Option<char>,
    escape_char: Option<char>,
    escape: bool,
}

impl<'a, T> ParseProcess<'a,  T> where T: TPeekable<Item=char>  {


    pub fn new(peekable: &mut T, stop_on:Option<char>,escape_char:Option<char>) -> ParseProcess< T> {
        ParseProcess { to_parse: peekable, current_position: 0, stop_on,escape_char, escape: false }
    }
    pub fn cur_pos(&self) -> usize {
        self.current_position
    }
}

impl<'a,  T> Iterator for ParseProcess<'a,  T> where T: TPeekable<Item=char> {
    type Item = char;
    fn next(&mut self) -> Option<char> {
        if !self.check_allowed() {
            return None;
        }
        let res = self.to_parse.next();
        if res.is_some() {
            self.current_position += 1
        }
        res
    }
}

impl<'a,  T> TPeekable for ParseProcess< 'a, T> where T: TPeekable<Item=char> {
    fn peek(&mut self) -> Option<&char> {
        if !self.check_allowed() {
            return None;
        }
        self.to_parse.peek()
    }

    fn peek_mut(&mut self) -> Option<&mut char> {
        self.to_parse.peek_mut()
    }
    fn next_if(&mut self, func: impl FnOnce(&char) -> bool) -> Option<char> {
        if !self.check_allowed() {
            return None;
        }
        let res = self.to_parse.next_if(func);
        if res.is_some() {
            self.current_position += 1
        }
        res
    }

    fn next_if_eq(&mut self, expected: &char) -> Option<char> {
        if !self.check_allowed() {
            return None;
        }
        let res = self.to_parse.next_if_eq(expected);
        if res.is_some() {
            self.current_position += 1
        }
        res
    }
}

impl< 'a, T> ParseProcess<'a, T> where T: TPeekable<Item=char> {
    fn check_allowed(&mut self) -> bool {
        let next = self.to_parse.peek();
        if let (Some(&c),Some(escape_char),Some(stop_char)) = (next, self.escape_char, self.stop_on) {
            if c == escape_char {
                self.escape = true;
            } else if c == stop_char && !self.escape {
                return false;
            } else {
                self.escape = false;
            }
        }
        true
    }
}

