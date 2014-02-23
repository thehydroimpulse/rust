// Copyright 2012 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

extern crate sync;
use sync::RWArc;
fn main() {
    let x = ~RWArc::new(1);
    let mut y = None; //~ ERROR lifetime of variable does not enclose its declaration
    x.write(|one| y = Some(one));
    *y.unwrap() = 2;
    //~^ ERROR lifetime of return value does not outlive the function call
    //~^^ ERROR dereference of reference outside its lifetime
}
