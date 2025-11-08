use std::fmt::Display;

use crate::*;

pub const MISSING_CLOSE_STRING: &str = "missing '}' to close a placeholder. If the '{' was intended to be a literal, escape it with '{{'";

#[derive(Clone)]
pub struct FormatStringParser<'a> {
    src: StrLitSlice<'a>,
    /// the characters in the source string
    chars: Vec<char>,
    /// The byte indices of the characters in the source string. Same length as `chars`.
    char_indices: Vec<usize>,
    /// Index into chars/char_indices of the next character to take
    pos: usize,
    /// Index of the most recent open curly bracket
    open_bracket_pos: usize,
}

impl<'a> FormatStringParser<'a> {
    pub fn new(src: StrLitSlice<'a>) -> Self {
        let (char_indices, chars) = src.text().char_indices().unzip();
        Self {
            src,
            chars,
            char_indices,
            pos: 0,
            open_bracket_pos: usize::MAX, // invalid value, will be set on first mark
        }
    }

    pub fn get_pos(&self) -> usize {
        self.pos
    }

    pub fn mark_open_bracket(&mut self, pos: usize) {
        self.open_bracket_pos = pos;
    }
    pub fn get_open_bracket_pos(&self) -> usize {
        self.open_bracket_pos
    }

    /// Take the next character from the source string, if available.
    pub fn take(&mut self) -> Result<(usize, char)> {
        let ret = self.peek_required()?;
        self.pos += 1;
        Ok(ret)
    }
    pub fn take_if(&mut self, f: impl FnOnce(char) -> bool) -> Option<(usize, char)> {
        let ret = self.peek()?;
        if !f(ret.1) {
            return None;
        }
        self.pos += 1;
        Some(ret)
    }
    pub fn take_if_eq(&mut self, c: char) -> Option<(usize, char)> {
        self.take_if(|x| x == c)
    }
    pub fn map_take_if<T>(&mut self, f: impl FnOnce(char) -> Option<T>) -> Option<(usize, T)> {
        let (pos, c) = self.peek()?;
        let ret = f(c)?;
        self.pos += 1;
        Some((pos, ret))
    }

    /// Return the next character without consuming it.
    pub fn peek(&mut self) -> Option<(usize, char)> {
        Some((self.pos, *self.chars.get(self.pos)?))
    }
    /// Return the next character without consuming it, returning an error if there is no next character.
    pub fn peek_required(&mut self) -> Result<(usize, char)> {
        match self.chars.get(self.pos) {
            Some(&c) => Ok((self.pos, c)),
            None => {
                if self.open_bracket_pos == usize::MAX {
                    // reached end without any placeholders, just return any error
                    Err(Error::new(Span::call_site(), ""))
                } else {
                    self.err_since(self.open_bracket_pos, MISSING_CLOSE_STRING)
                }
            }
        }
    }
    pub fn peek2(&mut self) -> Option<(usize, char)> {
        let next_pos = self.pos + 1;
        Some((next_pos, *self.chars.get(next_pos)?))
    }

    pub fn slice(&self, start: usize, end: usize) -> StrLitSlice<'a> {
        let start = self.char_indices[start];
        if let Some(end) = self.char_indices.get(end) {
            self.src.slice(start..*end)
        } else {
            self.src.slice(start..)
        }
    }
    pub fn slice_since(&self, start: usize) -> StrLitSlice<'a> {
        self.slice(start, self.pos)
    }

    pub fn err_since<T>(&self, start: usize, message: impl Display) -> Result<T> {
        self.slice_since(start).err(message)
    }
    pub fn err_at<T>(&self, pos: usize, message: impl Display) -> Result<T> {
        self.slice(pos, pos + 1).err(message)
    }
}
