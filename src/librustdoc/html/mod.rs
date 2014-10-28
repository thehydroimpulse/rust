// Copyright 2013-2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Rustdoc's HTML Rendering module
//!
//! This modules contains the bulk of the logic necessary for rendering a
//! rustdoc `clean::Crate` instance to a set of static HTML pages. This
//! rendering process is largely driven by the `format!` syntax extension to
//! perform all I/O into files and streams.
//!
//! The rendering process is largely driven by the `Context` and `Cache`
//! structures. The cache is pre-populated by crawling the crate in question,
//! and then it is shared among the various rendering tasks. The cache is meant
//! to be a fairly large structure not implementing `Clone` (because it's shared
//! among tasks). The context, however, should be a lightweight structure. This
//! is cloned per-task and contains information about what is currently being
//! rendered.
//!
//! In order to speed up rendering (mostly because of markdown rendering), the
//! rendering process has been parallelized. This parallelization is only
//! exposed through the `crate` method on the context, and then also from the
//! fact that the shared cache is stored in TLS (and must be accessed as such).
//!
//! In addition to rendering the crate itself, this module is also responsible
//! for creating the corresponding search index and source file renderings.
//! These tasks are not parallelized (they haven't been a bottleneck yet), and
//! both occur before the crate is rendered.

use renderer::Renderer;
use clean::Crate;
use std::path::Path;
use std::io::IoResult;

use std::collections::{HashMap, HashSet};
use std::collections::hashmap::{Occupied, Vacant};
use std::fmt;
use std::io::fs::PathExtensions;
use std::io::{fs, File, BufferedWriter, MemWriter, BufferedReader};
use std::io;
use std::str;
use std::string::String;
use std::sync::Arc;

use externalfiles::ExternalHtml;

use serialize::json;
use serialize::Encodable;
use serialize::json::ToJson;
use syntax::ast;
use syntax::ast_util;
use rustc::util::nodemap::NodeSet;

use clean;
use doctree;
use fold::DocFolder;
use format::{VisSpace, Method, FnStyleSpace, Stability};
use format::{ConciseStability, WhereClause};
use highlight;
use item_type::{ItemType, shortty};
use item_type;
use layout;
use markdown::Markdown;
use markdown;
use stability_summary;

pub mod highlight;
pub mod escape;
pub mod item_type;
pub mod format;
pub mod layout;
pub mod markdown;
pub mod toc;

/// Major driving force in all rustdoc rendering. This contains information
/// about where in the tree-like hierarchy rendering is occurring and controls
/// how the current page is being rendered.
///
/// It is intended that this context is a lightweight object which can be fairly
/// easily cloned because it is cloned per work-job (about once per item in the
/// rustdoc tree).
#[deriving(Clone)]
pub struct Context {
    /// Current hierarchy of components leading down to what's currently being
    /// rendered
    pub current: Vec<String>,
    /// String representation of how to get back to the root path of the 'doc/'
    /// folder in terms of a relative URL.
    pub root_path: String,
    /// This describes the layout of each page, and is not modified after
    /// creation of the context (contains info like the favicon and added html).
    pub layout: layout::Layout,
    /// This map is a list of what should be displayed on the sidebar of the
    /// current page. The key is the section header (traits, modules,
    /// functions), and the value is the list of containers belonging to this
    /// header. This map will change depending on the surrounding context of the
    /// page.
    pub sidebar: HashMap<String, Vec<String>>,
    /// This flag indicates whether [src] links should be generated or not. If
    /// the source files are present in the html rendering, then this will be
    /// `true`.
    pub include_sources: bool,
    /// A flag, which when turned off, will render pages which redirect to the
    /// real location of an item. This is used to allow external links to
    /// publicly reused items to redirect to the right location.
    pub render_redirect_pages: bool,
}

/// Indicates where an external crate can be found.
pub enum ExternalLocation {
    /// Remote URL root of the external crate
    Remote(String),
    /// This external crate can be found in the local doc/ folder
    Local,
    /// The external crate could not be found.
    Unknown,
}

/// Metadata about an implementor of a trait.
pub struct Implementor {
    def_id: ast::DefId,
    generics: clean::Generics,
    trait_: clean::Type,
    for_: clean::Type,
    stability: Option<clean::Stability>,
}

/// Metadata about implementations for a type.
#[deriving(Clone)]
pub struct Impl {
    impl_: clean::Impl,
    dox: Option<String>,
    stability: Option<clean::Stability>,
}

/// This cache is used to store information about the `clean::Crate` being
/// rendered in order to provide more useful documentation. This contains
/// information like all implementors of a trait, all traits a type implements,
/// documentation for all known traits, etc.
///
/// This structure purposefully does not implement `Clone` because it's intended
/// to be a fairly large and expensive structure to clone. Instead this adheres
/// to `Send` so it may be stored in a `Arc` instance and shared among the various
/// rendering tasks.
pub struct Cache {
    /// Mapping of typaram ids to the name of the type parameter. This is used
    /// when pretty-printing a type (so pretty printing doesn't have to
    /// painfully maintain a context like this)
    pub typarams: HashMap<ast::DefId, String>,

    /// Maps a type id to all known implementations for that type. This is only
    /// recognized for intra-crate `ResolvedPath` types, and is used to print
    /// out extra documentation on the page of an enum/struct.
    ///
    /// The values of the map are a list of implementations and documentation
    /// found on that implementation.
    pub impls: HashMap<ast::DefId, Vec<Impl>>,

    /// Maintains a mapping of local crate node ids to the fully qualified name
    /// and "short type description" of that node. This is used when generating
    /// URLs when a type is being linked to. External paths are not located in
    /// this map because the `External` type itself has all the information
    /// necessary.
    pub paths: HashMap<ast::DefId, (Vec<String>, ItemType)>,

    /// Similar to `paths`, but only holds external paths. This is only used for
    /// generating explicit hyperlinks to other crates.
    pub external_paths: HashMap<ast::DefId, Vec<String>>,

    /// This map contains information about all known traits of this crate.
    /// Implementations of a crate should inherit the documentation of the
    /// parent trait if no extra documentation is specified, and default methods
    /// should show up in documentation about trait implementations.
    pub traits: HashMap<ast::DefId, clean::Trait>,

    /// When rendering traits, it's often useful to be able to list all
    /// implementors of the trait, and this mapping is exactly, that: a mapping
    /// of trait ids to the list of known implementors of the trait
    pub implementors: HashMap<ast::DefId, Vec<Implementor>>,

    /// Cache of where external crate documentation can be found.
    pub extern_locations: HashMap<ast::CrateNum, ExternalLocation>,

    /// Cache of where documentation for primitives can be found.
    pub primitive_locations: HashMap<clean::PrimitiveType, ast::CrateNum>,

    /// Set of definitions which have been inlined from external crates.
    pub inlined: HashSet<ast::DefId>,

    // Private fields only used when initially crawling a crate to build a cache

    stack: Vec<String>,
    parent_stack: Vec<ast::DefId>,
    search_index: Vec<IndexItem>,
    privmod: bool,
    public_items: NodeSet,

    // In rare case where a structure is defined in one module but implemented
    // in another, if the implementing module is parsed before defining module,
    // then the fully qualified name of the structure isn't presented in `paths`
    // yet when its implementation methods are being indexed. Caches such methods
    // and their parent id here and indexes them at the end of crate parsing.
    orphan_methods: Vec<(ast::NodeId, clean::Item)>,
}

/// Helper struct to render all source code to HTML pages
struct SourceCollector<'a> {
    cx: &'a mut Context,

    /// Processed source-file paths
    seen: HashSet<String>,
    /// Root destination to place all HTML output into
    dst: Path,
}

/// Wrapper struct to render the source code of a file. This will do things like
/// adding line numbers to the left-hand side.
struct Source<'a>(&'a str);

// Helper structs for rendering items/sidebars and carrying along contextual
// information

struct Item<'a> { cx: &'a Context, item: &'a clean::Item, }
struct Sidebar<'a> { cx: &'a Context, item: &'a clean::Item, }

/// Struct representing one entry in the JS search index. These are all emitted
/// by hand to a large JS file at the end of cache-creation.
struct IndexItem {
    ty: ItemType,
    name: String,
    path: String,
    desc: String,
    parent: Option<ast::DefId>,
}

// TLS keys used to carry information around during rendering.

local_data_key!(pub cache_key: Arc<Cache>)
local_data_key!(pub current_location_key: Vec<String> )

pub struct HtmlRenderer {
    krate: Crate,
    cx: Context
}

impl HtmlRenderer {
    pub fn new(krate: Crate) -> HtmlRenderer {
        let crate_name = krate.name.clone();
        HtmlRenderer {
            krate: krate,
            cx: Context {
                current: Vec::new(),
                root_path: String::new(),
                sidebar: HashMap::new(),
                layout: layout::Layout::new(
                    "".to_string(),
                    "".to_string(),
                    crate_name,
                    "".to_string()
                ),
                include_sources: true,
                render_redirect_pages: false
            }
        }
    }
}

impl Renderer for HtmlRenderer {
    fn render(&mut self, dest: Path) -> IoResult<()> {

        try!(mkdir(&dest));

        // Crawl the crate, building a summary of the stability levels.  NOTE: this
        // summary *must* be computed with the original `krate`; the folding below
        // removes the impls from their modules.
        let summary = stability_summary::build(&self.krate);

        // Crawl the crate attributes looking for attributes which control how we're
        // going to emit HTML
        let default: &[_] = &[];
        match self.krate.module.as_ref().map(|m| m.doc_list().unwrap_or(default)) {
            Some(attrs) => {
                for attr in attrs.iter() {
                    match *attr {
                        clean::NameValue(ref x, ref s)
                                if "html_favicon_url" == x.as_slice() => {
                            self.cx.layout.favicon = s.to_string();
                        }
                        clean::NameValue(ref x, ref s)
                                if "html_logo_url" == x.as_slice() => {
                            self.cx.layout.logo = s.to_string();
                        }
                        clean::NameValue(ref x, ref s)
                                if "html_playground_url" == x.as_slice() => {
                            self.cx.layout.playground_url = s.to_string();
                            let name = self.krate.name.clone();
                            if markdown::playground_krate.get().is_none() {
                                markdown::playground_krate.replace(Some(Some(name)));
                            }
                        }
                        clean::Word(ref x)
                                if "html_no_source" == x.as_slice() => {
                            self.cx.include_sources = false;
                        }
                        _ => {}
                    }
                }
            }
            None => {}
        }

        // Crawl the crate to build various caches used for the output
        let analysis = ::analysiskey.get();
        let public_items = analysis.as_ref().map(|a| a.public_items.clone());
        let public_items = public_items.unwrap_or(NodeSet::new());
        let paths: HashMap<ast::DefId, (Vec<String>, ItemType)> =
          analysis.as_ref().map(|a| {
            let paths = a.external_paths.borrow_mut().take().unwrap();
            paths.into_iter().map(|(k, (v, t))| {
                (k, (v, match t {
                    clean::TypeStruct => item_type::Struct,
                    clean::TypeEnum => item_type::Enum,
                    clean::TypeFunction => item_type::Function,
                    clean::TypeTrait => item_type::Trait,
                    clean::TypeModule => item_type::Module,
                    clean::TypeStatic => item_type::Static,
                    clean::TypeVariant => item_type::Variant,
                    clean::TypeTypedef => item_type::Typedef,
                }))
            }).collect()
        }).unwrap_or(HashMap::new());
        let mut cache = Cache {
            impls: HashMap::new(),
            external_paths: paths.iter().map(|(&k, v)| (k, v.ref0().clone()))
                                 .collect(),
            paths: paths,
            implementors: HashMap::new(),
            stack: Vec::new(),
            parent_stack: Vec::new(),
            search_index: Vec::new(),
            extern_locations: HashMap::new(),
            primitive_locations: HashMap::new(),
            privmod: false,
            public_items: public_items,
            orphan_methods: Vec::new(),
            traits: analysis.as_ref().map(|a| {
                a.external_traits.borrow_mut().take().unwrap()
            }).unwrap_or(HashMap::new()),
            typarams: analysis.as_ref().map(|a| {
                a.external_typarams.borrow_mut().take().unwrap()
            }).unwrap_or(HashMap::new()),
            inlined: analysis.as_ref().map(|a| {
                a.inlined.borrow_mut().take().unwrap()
            }).unwrap_or(HashSet::new()),
        };
        cache.stack.push(self.krate.name.clone());
        self.krate = cache.fold_crate(self.krate);

        // Cache where all our extern crates are located
        for &(n, ref e) in self.krate.externs.iter() {
            cache.extern_locations.insert(n, extern_location(e, &cx.dst));
            let did = ast::DefId { krate: n, node: ast::CRATE_NODE_ID };
            cache.paths.insert(did, (vec![e.name.to_string()], item_type::Module));
        }

        // Cache where all known primitives have their documentation located.
        //
        // Favor linking to as local extern as possible, so iterate all crates in
        // reverse topological order.
        for &(n, ref e) in self.krate.externs.iter().rev() {
            for &prim in e.primitives.iter() {
                cache.primitive_locations.insert(prim, n);
            }
        }
        for &prim in self.krate.primitives.iter() {
            cache.primitive_locations.insert(prim, ast::LOCAL_CRATE);
        }

        // Build our search index
        let index = try!(build_index(&self.krate, &mut cache));

        // Freeze the cache now that the index has been built. Put an Arc into TLS
        // for future parallelization opportunities
        let cache = Arc::new(cache);
        cache_key.replace(Some(cache.clone()));
        current_location_key.replace(Some(Vec::new()));

        try!(write_shared(&self.cx, &self.krate, &*cache, index));
        let krate = try!(render_sources(&mut self.cx, self.krate));

        // And finally render the whole crate's documentation
        self.cx.krate(self.krate, summary)
    }
}
