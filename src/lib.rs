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

use itertools::Itertools;
use lsp_types::{Range, Url};
use parser::{DocumentTree, Rule, Search};

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

/// 定義済みのコマンドなど。
#[derive(Debug)]
pub struct Environment {
    /// インラインコマンド
    inline_cmds: Vec<InlineCmd>,
    /// ブロックコマンド
    block_cmds: Vec<BlockCmd>,
    /// 数式コマンド
    math_cmds: Vec<MathCmd>,
}

impl Environment {
    /// 新たな environment を作成する。
    fn new(doc: &DocumentTree<'_>) -> Self {
        let inline_cmds = doc
            .pickup(Rule::let_inline_stmt)
            .into_iter()
            .map(|pair| {
                let name = {
                    let mut children = pair.into_inner();
                    let fst = children.next().unwrap();
                    if fst.as_rule() == Rule::inline_cmd_name {
                        // let-inline \cmd の形
                        fst.as_str()
                    } else {
                        // let-inline ctx \cmd の形
                        let scd = children.next().unwrap();
                        scd.as_str()
                    }
                    .to_owned()
                };
                InlineCmd { name }
            })
            .collect_vec();
        let block_cmds = doc
            .pickup(Rule::let_block_stmt)
            .into_iter()
            .map(|pair| {
                let name = {
                    let mut children = pair.into_inner();
                    let fst = children.next().unwrap();
                    if fst.as_rule() == Rule::block_cmd_name {
                        // let-inline \cmd の形
                        fst.as_str()
                    } else {
                        // let-inline ctx \cmd の形
                        let scd = children.next().unwrap();
                        scd.as_str()
                    }
                    .to_owned()
                };
                BlockCmd { name }
            })
            .collect_vec();
        let math_cmds = doc
            .pickup(Rule::let_math_stmt)
            .into_iter()
            .map(|pair| {
                let name = pair.into_inner().next().unwrap().as_str();
                MathCmd {
                    name: name.to_owned(),
                }
            })
            .collect_vec();
        Environment {
            inline_cmds,
            block_cmds,
            math_cmds,
        }
    }
}

/// インラインコマンド。
#[derive(Debug)]
pub struct InlineCmd {
    /// コマンド名
    name: String,
    // def_range: Range,
}

/// ブロックコマンド。
#[derive(Debug)]
pub struct BlockCmd {
    /// コマンド名
    name: String,
    // def_range: Range,
}

/// 数式コマンド。
#[derive(Debug)]
pub struct MathCmd {
    /// コマンド名
    name: String,
    // def_range: Range,
}
