// Copyright 2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Generator provides the tooling for static site generation within Rust.
//! this forms as the backbone for rustdoc.
//!
//! List of major features that the generator supports:
//!
//!   * Frontmatter: A yaml-like format (except it's not yaml!)
//!   * Layouts: Ability to have custom layouts and specify what layout
//!              each file will follow.
//!   * Filters: Filters can be one of the following options:
//!                 * Registered based on some extension(s). (e.g., ".md")
//!                 * Registered based on some other filter. This
//!                   forms a dependency on a previously applied filter. This
//!                   is mainly used for the Markdown Validator.
//!   * Assets: These will be copied to the output directory.
//!   * Template Engine: An erb-like templating engine that powers the
//!                      layouts.
//!
//! Usage:
//!
//! The first task to to create a new generator:
//!
//! ```rust
//! extern crate generator;
//!
//! use generator::Generator;
//! use std::path::Path;
//!
//! fn main() {
//!   let mut gen = Generator::new(&Path::new("./cwd"));
//! }
//! ```
//!
//! With a generator, you can register filters, configure settings, and 
//! run the generator.
//!
//! ```rust
//! gen.run(&Path::new("./output"));
//! ```
//!
//! If `output` doesn't exist, the folder will be created, along with
//! the generated resources.
//!
//! Here's the default directory structure of a static site:
//!
//! ```notrust
//! content/
//!   + hello.md
//! layouts/
//!   + index.html
//!   + docs.html
//! assets/
//!   images/
//!     + logo.png
//!   stylesheets/
//!     + index.css
//!   javascripts/
//!     + index.js
//! output/ # Generated site that can be served as is.
//! ```
//!


#![crate_id = "generator#0.11.0-pre"]
#![crate_type = "rlib"]
#![crate_type = "dylib"]
#![license = "MIT/ASL2"]
#![doc(html_logo_url = "http://www.rust-lang.org/logos/rust-logo-128x128-blk-v2.png",
       html_favicon_url = "http://www.rust-lang.org/favicon.ico",
       html_root_url = "http://static.rust-lang.org/doc/master")]
#![feature(globs, phase, macro_rules)]
//#![deny(missing_doc)]
#![deny(deprecated_owned_vector)]

#[cfg(test)] #[phase(syntax, link)] extern crate log;
extern crate collections;
extern crate serialize;
extern crate regex;
#[phase(syntax)]
extern crate regex_macros;

pub mod layout;
pub mod page;
pub mod filter;
pub mod template;
pub mod frontmatter;
pub mod result;
pub mod generator;

pub type Generator<'a> = generator::Generator<'a>;
