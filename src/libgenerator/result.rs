// Copyright 2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use serialize::json;
use std::io;

pub fn io_error(io: io::IoError) -> GeneratorError {
    GeneratorError {
        kind: IoError(io.clone()),
        description: StaticDescription(io.desc)
    }
}

pub fn decoder_error(decode: json::DecoderError) -> GeneratorError {
    let desc = match decode.clone() {
        json::ParseError(parse) => { StrBuf::new() },
        json::ExpectedError(one, two) => { StrBuf::new() },
        json::MissingFieldError(s) => s,
        json::UnknownVariantError(s) => s
    };

    GeneratorError {
        kind: DecodeError(decode),
        description: BoxedDescription(desc)
    }
}

pub struct GeneratorError {
    kind: ErrorKind,
    description: ErrorDescription
}

pub enum ErrorDescription {
    BoxedDescription(StrBuf),
    StaticDescription(&'static str)
}

pub enum ErrorKind {
    Unknown,
    MalformedFrontmatter,
    IoError(io::IoError),
    DecodeError(json::DecoderError)
}

pub type GeneratorResult<T> = Result<T, GeneratorError>;

