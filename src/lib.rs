//! Maquette (prototype) of SATySFi Language Server.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

#[macro_use]
extern crate pest_derive;

pub mod completion;
pub mod definition;
pub mod parser;

use anyhow::{Error, Result};
use log::debug;
use pest::{Parser, Span};

use std::collections::HashMap;

use itertools::Itertools;
use lsp_types::{Position, Range, Url};
use parser::{Mode, Pair, Rule, SatysfiParser};

#[derive(Debug)]
pub struct Buffer {
    pub buf_cst: BufferCst,
    pub error: Vec<Error>,
    pub env: Environment,
}

/// Cst を格納した Buffer.
#[derive(Debug, Clone)]
pub struct BufferCst {
    /// バッファの文字列本体。
    pub buffer: String,
    /// バッファの文法構造。
    cst: Option<Cst>,
}

impl Buffer {
    /// 与えられた文字列を消費し、新たな Buffer を作成する。
    pub fn new(text: String) -> Self {
        let (text, e) = BufferCst::parse_into(text);
        let error = e.into_iter().collect_vec();
        let env = Environment::new(&text);

        Self { buf_cst: text, error, env }
    }
}

impl BufferCst {
    /// 与えられた文字列を消費し、新たな BufferCst を作成する。
    pub fn parse_into(buffer: String) -> (Self, Option<Error>) {
        let pairs = SatysfiParser::parse(Rule::program, &buffer);
        match pairs {
            Ok(mut pairs) => {
                let pair = pairs.next().unwrap();
                let cst = Some(Cst::from(pair));
                (Self { buffer, cst }, None)
            }
            Err(e) => {
                let error = Error::from(e);
                (Self { buffer, cst: None }, Some(error))
            }
        }
    }

    /// Cst の示す部分文字列を返す。
    /// Cst の range が UTF-8 として正しい文字列であることを前提とする
    /// （そうなっていなければ panic する）。
    pub fn as_str(&self, cst: &Cst) -> &str {
        cst.as_str(&self.buffer)
    }
}

impl std::fmt::Display for BufferCst {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.cst {
            Some(cst) => {
                let text = cst.pretty_text(&self.buffer, 0);
                write!(f, "{}", text)
            },
            None => {
                write!(f, r#"No CST. raw str: """
{}
""""#, self.buffer)
            },
        }
    }
}

/// 参照をなくして BufferCst が自己参照構造体になることを回避した
/// pest::iterators::Pair 的なもの。再帰構造を持つ。
#[derive(Debug, Clone)]
pub struct Cst {
    /// そのルールが何であるか。
    rule: Rule,
    /// Cst が表す範囲。
    range: CstRange,
    /// 子 Cst。
    inner: Vec<Cst>,
}

impl<'a> From<Pair<'a>> for Cst {
    fn from(pair: Pair<'a>) -> Self {
        let rule = pair.as_rule();
        let range = CstRange::from(pair.as_span());
        let inner = pair.into_inner().map(Cst::from).collect_vec();
        Self { rule, range, inner }
    }
}

/// ルールで検索したり、ある位置を含む Pair を探索したりできるもの。
impl Cst {
    /// 与えられたルールの Cst を再帰的に抽出する。
    fn pickup(&self, rule: Rule) -> Vec<&Cst> {
        let mut vec = vec![];
        for cst in &self.inner {
            if cst.rule == rule {
                vec.push(cst)
            }
            let v = cst.pickup(rule);
            vec.extend(v);
        }
        vec
    }

    /// 自分の子のうち、与えられた pos を含むものを返す。
    fn choose(&self, pos: &Position) -> Option<&Cst> {
        for cst in &self.inner {
            if cst.range.includes(pos) {
                return Some(cst);
            }
        }
        None
    }

    /// 与えられた pos を含む Pair を再帰的に探索する。
    fn dig(&self, pos: &Position) -> Vec<&Cst> {
        let child = self.choose(pos);
        if let Some(child) = child {
            let mut v = child.dig(pos);
            v.push(child);
            v
        } else {
            vec![]
        }
    }

    /// Cst の構造を箇条書き形式で出力する。
    fn pretty_text(&self, text: &str, indent: usize) -> String {
        let content = if self.inner.len() == 0 {
            format!(
                "| [{rule:?}] ({sl}:{sc}..{el}:{ec}): \"{text}\"\n",
                rule = self.rule,
                sl = self.range.start.line,
                sc = self.range.start.character,
                el = self.range.end.line,
                ec = self.range.end.character,
                text = self.as_str(text)
            )
        } else {
            let children = self.inner.iter().map(|cst| {
                cst.pretty_text(text, indent + 2)
            }).join("");
            format!(
                "- [{rule:?}] ({sl}:{sc}..{el}:{ec})\n{children}",
                rule = self.rule,
                sl = self.range.start.line,
                sc = self.range.start.character,
                el = self.range.end.line,
                ec = self.range.end.character,
                children = children
            )
        };

        format!("{indent}{content}", indent = " ".repeat(indent), content = content)
    }

    fn as_str<'a>(&self, text: &'a str) -> &'a str {
        let start = self.range.start.byte;
        let end = self.range.end.byte;
        std::str::from_utf8(&text.as_bytes()[start..end]).unwrap()
    }

    fn mode(&self, pos: &Position) -> Mode {
        let csts = self.dig(pos);
        let rules = csts.iter().map(|cst| cst.rule);

        for rule in rules {
            match rule {
                Rule::vertical_mode => return Mode::Vertical,
                Rule::horizontal_mode => return Mode::Horizontal,
                Rule::math_mode => return Mode::Math,
                Rule::headers | Rule::header_stage => return Mode::Header,
                Rule::COMMENT => return Mode::Comment,
                Rule::string_interior => return Mode::Literal,
                Rule::cmd_expr_arg
                | Rule::cmd_expr_option
                | Rule::math_cmd_expr_arg
                | Rule::math_cmd_expr_option => return Mode::Program,
                _ => continue,
            }
        }
        Mode::Program
    }
}

#[derive(Debug, Clone)]
pub struct CstRange {
    /// 始まりの位置。
    start: CstPosition,
    /// 終わりの位置。
    end: CstPosition,
}

impl<'a> From<Span<'a>> for CstRange {
    fn from(span: Span<'a>) -> Self {
        let start = CstPosition::from(span.start_pos());
        let end = CstPosition::from(span.end_pos());
        Self { start, end }
    }
}

impl Into<lsp_types::Range> for CstRange {
    fn into(self) -> lsp_types::Range {
        lsp_types::Range {
            start: self.start.into(),
            end: self.end.into(),
        }
    }
}

impl CstRange {
    fn includes(&self, pos: &Position) -> bool {
        let start: Position = self.start.clone().into();
        let end: Position = self.end.clone().into();
        pos >= &start && pos <= &end
    }
}

#[derive(Debug, Clone)]
pub struct CstPosition {
    /// スタートから何バイト目にあるか。
    byte: usize,
    /// 何行目にあるか。
    line: u32,
    /// その行の何文字目にあるか。
    character: u32,
}

impl<'a> From<pest::Position<'a>> for CstPosition {
    fn from(pos: pest::Position<'a>) -> Self {
        let byte = pos.pos();
        let (line, character) = pos.line_col();
        let line = (line - 1) as u32;
        let character = (character - 1) as u32;
        Self {
            byte,
            line,
            character,
        }
    }
}

impl Into<lsp_types::Position> for CstPosition {
    fn into(self) -> lsp_types::Position {
        lsp_types::Position {
            line: self.line,
            character: self.character,
        }
    }
}

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
#[derive(Debug, Default)]
pub struct Environment {
    /// インラインコマンド
    inline_cmds: Vec<InlineCmd>,
    /// ブロックコマンド
    block_cmds: Vec<BlockCmd>,
    /// 数式コマンド
    math_cmds: Vec<MathCmd>,
    /// let 式で定義された変数
    variables: Vec<Variable>,
}

impl Environment {
    /// 新たな environment を作成する。
    fn new(text: &BufferCst) -> Self {
        match &text.cst {
            None => Environment::default(),
            Some(cst) => {
                let inline_cmds = cst
                    .pickup(Rule::let_inline_stmt)
                    .into_iter()
                    .map(|cst| {
                        let mut children = cst.inner.iter();
                        let fst = children.next().unwrap();
                        if fst.rule == Rule::inline_cmd_name {
                            // let-inline \cmd の形
                            let name = text.as_str(fst).to_owned();
                            let def_range = fst.range.clone().into();
                            InlineCmd {name, def_range}
                        } else {
                            // let-inline ctx \cmd の形
                            let scd = children.next().unwrap();
                            let name = text.as_str(scd).to_owned();
                            let def_range = scd.range.clone().into();
                            InlineCmd {name, def_range}
                        }
                    })
                    .collect_vec();

                let block_cmds = cst
                    .pickup(Rule::let_block_stmt)
                    .into_iter()
                    .map(|cst| {
                            let mut children = cst.inner.iter();
                            let fst = children.next().unwrap();
                            if fst.rule == Rule::block_cmd_name {
                                // let-block +cmd の形
                            let name = text.as_str(fst).to_owned();
                            let def_range = fst.range.clone().into();
                            BlockCmd {name, def_range}
                            } else {
                                // let-block ctx +cmd の形
                                let scd = children.next().unwrap();
                                let name = text.as_str(scd).to_owned();
                                let def_range = scd.range.clone().into();
                                BlockCmd {name, def_range}
                            }
                    })
                    .collect_vec();

                let math_cmds = cst
                    .pickup(Rule::let_math_stmt)
                    .into_iter()
                    .map(|cst| {
                        let mut children = cst.inner.iter();
                        let fst = children.next().unwrap();
                        let name = text.as_str(fst).to_owned();
                        let def_range = fst.range.clone().into();
                        MathCmd { name, def_range }
                    })
                    .collect_vec();

                let variables = cst
                    .pickup(Rule::let_stmt)
                    .into_iter()
                    .map(|cst| {
                        let mut children = cst.inner.iter();
                        let ptn = children.next().unwrap();
                        ptn.pickup(Rule::var).into_iter().map(|cst| {
                            let name = text.as_str(cst).to_owned();
                            let def_range = cst.range.clone().into();
                            Variable{ name, def_range }
                        })
                    }).flatten()
                .collect_vec();

                Self { inline_cmds, block_cmds, math_cmds, variables }
            }
        }

    }
}

/// インラインコマンド。
#[derive(Debug)]
pub struct InlineCmd {
    /// コマンド名
    name: String,
    /// 定義の場所
    def_range: Range,
}

/// ブロックコマンド。
#[derive(Debug)]
pub struct BlockCmd {
    /// コマンド名
    name: String,
    /// 定義の場所
    def_range: Range,
}

/// 数式コマンド。
#[derive(Debug)]
pub struct MathCmd {
    /// コマンド名
    name: String,
    /// 定義の場所
    def_range: Range,
}

/// 変数
#[derive(Debug)]
pub struct Variable {
    /// 変数名
    name: String,
    /// 定義の場所
    def_range: Range,
}
