//! Maquette (prototype) of SATySFi Language Server.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(rust_2018_idioms)]
#![warn(clippy::missing_docs_in_private_items)]

#[macro_use]
extern crate pest_derive;

pub mod completion;
pub mod parser;

use std::collections::HashMap;

use lsp_types::Url;

/// バッファを格納する map.
#[derive(Debug, Default)]
pub struct Buffers {
    /// URL に対応するテキスト（バッファの中身）の内容。
    texts: HashMap<Url, String>,
}

impl Buffers {
    /// get text from Buffers.
    pub fn get(&self, uri: &Url) -> Option<&str> {
        self.texts.get(uri).map(|s| s.as_str())
    }

    /// set (register) text to Buffers.
    pub fn set(&mut self, uri: Url, text: String) {
        self.texts.insert(uri, text);
    }
}
