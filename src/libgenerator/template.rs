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
use regex::{Captures, Regex};

/// An erb-style templating system. This stores the
/// AST of the template.
///
/// ```html
/// <!DOCTYPE html>
/// <html>
///     <head>
///         <title><%= title %></title>
///     </head>
/// </html>
/// ```
///
/// The rules are fairly simple. The interpolation starts with a `<%` and `=` signifies
/// the output will be printed. Within the containment, a single identifier must be found.
///
/// Usage:
///
/// ```rust
/// let mut template = Template::new(r"<%= foobar %>");
///
/// // Parse the template:
/// template.parse();
///
/// // Render it
/// template
///     .context()
///     .add("foobar", "fah")
///     .render();
/// ```
pub struct Template<'a, 't> {
    input: &'a str,
    reg: Regex
}

impl<'a, 't> Template<'a, 't> {
    pub fn new(input: &'a str) -> Template<'a, 't> {
        Template {
            input: input,
            reg: regex!(r"(?P<interp><%= (?P<var>[A-Za-z][A-Za-z0-9_]+) %>)+?")
        }
    }

    pub fn render(&'a mut self, context: HashMap<StrBuf, StrBuf>) -> Result<StrBuf, StrBuf> {

        let result = self.reg.replace_all(self.input, |caps: &Captures| {
            let name = caps.name("var").to_strbuf();
            let var  = context.find(&name).expect(format!("Templating Error:
                Variable `{}` was not found in the current context.", name));
            format_strbuf!("{}", var)
        });

        Ok(result)
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use collections::hashmap::HashMap;


    #[test]
    fn foobar() {
        let mut template = Template::new(r"<%= foobar %>");

        let mut context = HashMap::new();
        context.insert("foobar".to_strbuf(), "bar".to_strbuf());

        assert_eq!(template.render(context).unwrap(), "bar".to_strbuf());
    }

    #[test]
    fn mix() {
        let mut template = Template::new(r"<%= foobar %> hahaha");

        let mut context = HashMap::new();
        context.insert("foobar".to_strbuf(), "bar".to_strbuf());

        assert_eq!(template.render(context).unwrap(), "bar hahaha".to_strbuf());
    }

    #[test]
    fn multiple_vars() {
        let mut template = Template::new(r"<%= title %> <%= foobar %>");

        let mut context = HashMap::new();
        context.insert("foobar".to_strbuf(), "bar".to_strbuf());
        context.insert("title".to_strbuf(), "two".to_strbuf());

        assert_eq!(template.render(context).unwrap(), "two bar".to_strbuf());
    }
}
