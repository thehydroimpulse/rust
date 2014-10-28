// Copyright 2013 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::fmt;
use std::io;

#[deriving(Clone)]
pub struct Layout {
    pub logo: String,
    pub favicon: String,
    pub krate: String,
    pub playground_url: String,
}

impl Layout {
    pub fn new(logo: String, favicon: String, krate: String, playground_url: String) -> Layout {
        Layout {
            logo: logo,
            favicon: favicon,
            krate: krate,
            playground_url: playground_url
        }
    }
}

pub struct Page<'a> {
    pub title: &'a str,
    pub ty: &'a str,
    pub root_path: &'a str,
    pub description: &'a str,
    pub keywords: &'a str
}

pub fn render<T: fmt::Show, S: fmt::Show>(
    dst: &mut io::Writer, layout: &Layout, page: &Page, sidebar: &S, t: &T)
    -> io::IoResult<()>
{
    write!(dst,
r##"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <meta name="generator" content="rustdoc">
    <meta name="description" content="{description}">
    <meta name="keywords" content="{keywords}">

    <title>{title}</title>

    <link rel="stylesheet" type="text/css" href="{root_path}main.css">

    {favicon}
    {in_header}
</head>
<body>
    <!--[if lte IE 8]>
    <div class="warning">
        This old browser is unsupported and will most likely display funky
        things.
    </div>
    <![endif]-->

    <div class="container">
      <header>
        <section>
          <a href="#">{logo} <span class="crate">Crate</span> {krate}</a>
        </section>
      </header>
      <section class="content">
        <div class="sidebar">
          {sidebar}
          <div class="group large">
            <div class="bar"></div>
            <span class="circle"></span>
            <span class="name">Docs</span>
            <div class="sub-group active">
              <span class="circle"></span>
              <span class="name"><a href="#">Introduction</a></span>
            </div>
          </div>
          <div class="group medium">
            <div class="bar"></div>
            <span class="circle"></span>
            <span class="name">Crates</span>
            <div class="sub-group stable" title="Stable">
              <span class="circle"></span>
              <span class="name"><a href="#">Gossip</a></span>
            </div>
            <div class="sub-group unstable" title="Unstable">
              <span class="circle"></span>
              <span class="name"><a href="#">Data</a></span>
            </div>
          </div>
          <div class="group small dependencies">
            <div class="bar"></div>
            <span class="circle"></span>
            <span class="name">Dependencies</span>
            <span class="expand">+</span>
            <div class="sub-group">
              <span class="circle"></span>
              <span class="name"><a href="#">Docopt</a></span>
            </div>
            <div class="sub-group">
              <span class="circle"></span>
              <span class="name"><a href="#">Teepee</a></span>
            </div>
            <div class="sub-group">
              <span class="circle"></span>
              <span class="name"><a href="#">Nanomsg</a></span>
            </div>
            <div class="sub-group">
              <span class="circle"></span>
              <span class="name"><a href="#">Uuid</a></span>
            </div>
            <div class="sub-group">
              <span class="circle"></span>
              <span class="name"><a href="#">Msgpack</a></span>
            </div>
          </div>
        </div>
        <div class="wrapper">
            {before_content}
            {content}
            {after_content}
        </div>
      </section>
    </div>

    <script>
        window.rootPath = "{root_path}";
        window.currentCrate = "{krate}";
        window.playgroundUrl = "{play_url}";
    </script>
    <script src="{root_path}jquery.js"></script>
    <!--<script src="{root_path}main.js"></script>-->
    {play_js}
    <!--<script async src="{root_path}search-index.js"></script>-->
</body>
</html>"##,
    content   = *t,
    root_path = page.root_path,
    logo      = if layout.logo.len() == 0 {
        "".to_string()
    } else {
        format!("<a class='logo' href='{}{}/index.html'>\
                 <img src='{}' alt='' width='30'></a>",
                page.root_path, layout.krate,
                layout.logo)
    },
    title     = page.title,
    sidebar = *sidebar,
    description = page.description,
    keywords = page.keywords,
    favicon   = if layout.favicon.len() == 0 {
        "".to_string()
    } else {
        format!(r#"<link rel="shortcut icon" href="{}">"#, layout.favicon)
    },
    in_header = layout.external_html.in_header,
    before_content = layout.external_html.before_content,
    after_content = layout.external_html.after_content,
    krate     = layout.krate,
    play_url  = layout.playground_url,
    play_js   = if layout.playground_url.len() == 0 {
        "".to_string()
    } else {
        format!(r#"<script src="{}playpen.js"></script>"#, page.root_path)
    },
    )
}

pub fn redirect(dst: &mut io::Writer, url: &str) -> io::IoResult<()> {
    write!(dst,
r##"<!DOCTYPE html>
<html lang="en">
<head>
    <meta http-equiv="refresh" content="0;URL={url}">
</head>
<body>
</body>
</html>"##,
    url = url,
    )
}
