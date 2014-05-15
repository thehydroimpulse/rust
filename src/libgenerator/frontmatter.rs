// Copyright 2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use collections::hashmap::HashMap;
use std::str::Chars;
use std::iter::{Peekable, range};

/// Each guide can contain some frontmatter that can add additional
/// metadata associated with the file itself. Traditionally, this is
/// a Yaml format, but, considering Yaml is an insanely complex standard, this
/// is going to be a simpler key-value store.
///
/// Example:
///
/// ```notrust
/// ---
/// title: "Foo bar"
/// category:
///     - "Foo"
///     - "Fah"
///     - "Fee"
/// ---
/// ```
pub struct Frontmatter<'a> {
    input: &'a str,
    pairs: HashMap<StrBuf, Types>,
    iter: Peekable<char, Chars<'a>>,
    state: State,
    current: Token
}

/// Types that are supported by the frontend key-value format. Yaml supports a **lot**
/// more than this implementation; we're simply sticking to simple types for now.
#[deriving(Eq,Show)]
pub enum Types {
    Integer(int),
    String(StrBuf)
}

/// The current state of the lexer. This allows us to easily track whether the lexer
/// is within a quoted string, number, etc...
#[deriving(Eq,Show)]
pub enum State {
    /// Parsing a double quote. We found the first one, and collecting everything
    /// in-between until we find another single quote.
    SDoubleQuote,
    /// Parsing a single quote. We found the first one, and collecting everything
    /// in-between until we find another single quote.
    SSingleQuote,
    /// Parsing a key. Rule: ^[A-Za-z][A-Za-z0-9_]+:
    SKey,
    /// The lexer is parsing the value of a key. The value can be of many different
    /// formats, so this requires some lookaheads.
    SValue,
    /// A None-alias. The lexer is in an idle state and not parsing anything
    /// specific.
    SIdle,
    STag
}

/// List of tokens that a frontend will contain. The lexer will throw a stream
/// of tokens that we have found in a particular input.
#[deriving(Eq,Show,Clone)]
pub enum Token {
    /// An identifier is similar to a string, but isn't contained within quotes and
    /// has more restrictions. [A-Za-z\$\%][A-Za-z0-9_]+ is the format identifiers are
    /// restricted to.
    TIdentifier(StrBuf),
    TColon,
    /// A single double quote. This typically won't be outputted, unless a malformed string
    /// is found.
    TDoubleQuote,
    /// The same applies to the single quote string.
    TSingleQuote,
    /// A string that was wrapped around either single or double quotes.
    TStr(StrBuf),
    /// `-`
    TDash,
    /// An integer. This represents a collection of single numbers.
    TInteger(int),
    /// None. Represents an empty/null value.
    TBlank,
    /// \n
    TLineBreak,
    /// Beginning of the frontmatter (i.e., the `---\n`)
    TBegin,
    /// The end of the frontmatter has been found. Parsing is done.
    TEnd,

    TTag
}

impl<'a> Frontmatter<'a> {

    /// Initialize a new Frontmatter. The frontmatter will not be parsed yet, that is an
    /// explicit step taken by the user.
    pub fn new(input: &'a str) -> Frontmatter<'a> {
        Frontmatter {
            input: input,
            pairs: HashMap::new(),
            iter: input.chars().peekable(),
            state: SIdle,
            current: TBlank
        }
    }

    pub fn parse(&mut self) -> Result<(), StrBuf> {

        // ---
        try!(self.parse_dashes(true));

        // ---
        try!(self.parse_dashes(false));

        Ok(())
    }

    pub fn parse_dashes(&mut self, line_break: bool) -> Result<(), StrBuf> {
        // Look for the beginning three tokens: "---" that sits on it's
        // own line.
        for i in range(0, 3) {
            let mut token = self.bump();

            // Ignore line breaks in this context.
            while i == 0 && token == TLineBreak {
                token = self.bump();
            }

            if token != TDash {
                return Err(format_strbuf!("Frontmatter Error: Expected `-`, but found {}", token));
            }
        }

        // Ensure that the dashes happened three times, followed by a line break.
        // Otherwise, we'll simply fail.
        if line_break && self.bump() != TLineBreak {
            return Err(format_strbuf!("Frontmatter Error: Expected a line break but found {}", self.current));
        }

        Ok(())
    }

    pub fn bump(&mut self) -> Token {
        let t = self.next_token(false);
        self.current = t.clone();
        t
    }

    pub fn peek(&mut self) -> Token {
        self.next_token(true)
    }

    pub fn next_char(&mut self, peek: bool) -> char {
        if peek {
            *self.iter.peek().unwrap()
        } else {
            self.iter.next().unwrap()
        }
    }

    pub fn next_token(&mut self, peek: bool) -> Token {

        return match self.next_char(peek) {
            ':' => TColon,
            '-' => {
                if self.state != STag {
                    self.state = STag;
                    let mut found = true;

                    for i in range(0, 3) {
                        let token = self.bump();
                        if token != TDash { found = false; break; }
                    }

                    if found {
                        TTag
                    } else {
                        TDash
                    }

                } else {
                    TDash
                }
            },
            '"' => {
                self.state = if self.state != SDoubleQuote {
                    SDoubleQuote
                } else {
                    SIdle
                };

                match self.peek() {
                    TStr(s) => TStr(s),
                    _ => TDoubleQuote
                }
            },
            ' ' => self.bump(),
            '\'' => TSingleQuote,
            '0' => TInteger(0),
            '1' => TInteger(1),
            '2' => TInteger(2),
            '3' => TInteger(3),
            '4' => TInteger(4),
            '5' => TInteger(5),
            '6' => TInteger(6),
            '7' => TInteger(7),
            '8' => TInteger(8),
            '9' => TInteger(9),
            '\n' => TLineBreak,
            c => {
                match self.state {
                    SDoubleQuote => {
                        let mut ch  = c;
                        let mut buf = StrBuf::new();

                        while ch != '"' {
                            buf.push_char(ch);
                            ch = self.next_char(peek);
                        }

                        TStr(buf)
                    },
                    SKey => {
                        let mut ch  = c;
                        let mut buf = StrBuf::new();

                        while ch != ':' {
                            buf.push_char(ch);
                            ch = self.next_char(peek);
                        }

                        TIdentifier(buf)
                    },
                    _ => TBlank
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn should_detect_frontmatter() {
        let mut frontmatter = Frontmatter::new("---\n---");
        frontmatter.parse().unwrap();

        frontmatter = Frontmatter::new(r"---
            ---");
        frontmatter.parse().unwrap();
    }

    #[test]
    fn should_accept_linebreaks() {
        let mut frontmatter = Frontmatter::new(r"---

            ---");

        frontmatter.parse().unwrap();
    }


    #[test]
    fn parse_string() {
        let mut frontmatter = Frontmatter::new(r"---
            key: 'foobar'
            ---");

        frontmatter.parse().unwrap();
    }
}
