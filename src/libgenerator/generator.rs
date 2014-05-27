// Copyright 2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::path::Path;
use std::io::fs::walk_dir;
use serialize::{json, Encodable, Decodable};
use std::io::fs::File;
use std::io;
use std::io::{IoResult};

use result::{GeneratorResult, io_error, decoder_error};
use layout::Layout;
use page::Page;
use filter::Filter;

#[deriving(Encodable, Eq, Decodable)]
pub struct ConfigJson {
    assets_path: Option<StrBuf>,
    content_path: Option<StrBuf>,
    layouts_path: Option<StrBuf>,
    output_path: Option<StrBuf>
}

pub struct Config {
    assets: Path,
    content: Path,
    layouts: Path,
    output: Path
}

pub struct Generator<'a> {
    layouts: Vec<Layout<'a>>,
    files: Vec<Page<'a>>,
    directory: Path,
    filters: Vec<Filter<'a>>,
    config: ConfigJson
}

impl Config {

    pub fn from_json<'a>(content: &'a str) -> GeneratorResult<Config> {
        let obj = json::from_str(content);
        let mut config = Config::default();
        let mut decoder = json::Decoder::new(obj.unwrap());
        let json: ConfigJson = try!(Decodable::decode(&mut decoder).map_err(decoder_error));

        match json.assets_path {
            Some(ref buf) => config.assets = Path::new(buf.as_slice()),
            None => {}
        }

        match json.content_path {
            Some(ref buf) => config.content = Path::new(buf.as_slice()),
            None => {}
        }

        match json.layouts_path {
            Some(ref buf) => config.layouts = Path::new(buf.as_slice()),
            None => {}
        }

        match json.output_path {
            Some(ref buf) => config.output = Path::new(buf.as_slice()),
            None => {}
        }

        Ok(config)
    }

    pub fn default() -> Config {
        Config {
            assets: Path::new("assets"),
            content: Path::new("content"),
            layouts: Path::new("layouts"),
            output: Path::new("output")
        }
    }
}

impl ConfigJson {

    pub fn default() -> ConfigJson {
        ConfigJson {
            assets_path: None,
            content_path: None,
            layouts_path: None,
            output_path: None
        }
    }
}

impl<'a> Generator<'a> {

    /// The path is the working directory to look inside. This includes assets, markdown files,
    /// assets, etc...
    pub fn new(path: Path) -> Generator<'a> {
        Generator {
            layouts: Vec::new(),
            files: Vec::new(),
            directory: path,
            filters: Vec::new(),
            config: ConfigJson::default()
        }
    }

    /// Lookup all the files and directories within the current folder. This will shove all
    /// the files within their appropriate vector and start reading in the files.
    ///
    /// Currently, the folder structure is static:
    ///
    /// ```notrust
    /// static/
    ///     + content/
    ///     + layouts/
    ///     + assets/
    ///     + output
    /// ```
    pub fn lookup(&mut self) -> GeneratorResult<()> {
        if !self.directory.exists() {
            fail!("The path specified {} doesn't exist.", self.directory.display());
        }

        for item in try!(walk_dir(&self.directory).map_err(io_error)) {
            let file = try!(File::open(&item).read_to_str().map_err(io_error));

            if item.as_str().unwrap().contains("config.json") {
                let config = try!(Config::from_json(file.as_slice()));
                println!("{:?}", config);
            }
            println!("{}", item.display());
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::path::Path;

    #[test]
    fn should_lookup() {
      let mut gen = Generator::new(Path::new("./src/libgenerator/mock"));
      gen.lookup();
      fail!("{}");
    }
}
