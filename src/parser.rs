//! SATySFi のパーサ。

/// SATySFi の PEG パーサ本体。
#[allow(missing_docs)]
mod satysfi_parser {
    #[derive(Parser)]
    #[grammar = "parser/satysfi.pest"]
    pub struct SatysfiParser;
}

pub mod relation;

use anyhow::Result;
use itertools::Itertools;
use log::debug;
use lsp_types::{Position, Range};
use pest::{Parser, Span};
pub use satysfi_parser::{Rule, SatysfiParser};

/// CalculatorParser で用いられる Pair.
pub type Pair<'i> = pest::iterators::Pair<'i, Rule>;

#[derive(Debug, PartialEq, Eq)]
pub enum Mode {
    /// プログラムモード。
    Program,
    /// 垂直モード。
    Vertical,
    /// 水平モード。
    Horizontal,
    /// 数式モード。
    Math,
    /// ヘッダ。
    Header,
    /// 文字列リテラル。
    Literal,
    /// コメント。
    Comment,
}

/// ルールで検索したり、ある位置を含む Pair を探索したりできるもの。
pub trait Search<'a> {
    /// 与えられたルールの Pair を全て抽出する。
    fn pickup(&self, rule: Rule) -> Vec<Pair<'a>>;

    /// 自分の子のうち、与えられた pos を含むものを返す。
    fn choose(&self, pos: &Position) -> Option<Pair<'a>>;

    /// 与えられた pos を含む Pair を再帰的に探索する。
    fn dig(&self, pos: &Position) -> Vec<Pair<'a>> {
        let child = self.choose(pos);
        if let Some(child) = child {
            let mut v = child.clone().dig(pos);
            v.push(child);
            v
        } else {
            vec![]
        }
    }
}

/// 文書の構文木。
#[derive(Debug)]
pub struct DocumentTree<'a> {
    /// 各行の Pair。空行は None。
    pub tree: std::result::Result<Pair<'a>, pest::error::Error<Rule>>,
}

impl<'a> DocumentTree<'a> {
    /// 文書から構文木を生成する。
    pub fn from_document(text: &'a str) -> Self {
        let tree = match SatysfiParser::parse(Rule::program, text) {
            Ok(mut pairs) => Ok(pairs.next().unwrap()),
            Err(e) => Err(e),
        };
        DocumentTree { tree }
    }

    /// カーソル位置のモードを出力する。不明のときは None を返す。
    pub fn mode(&self, pos: &Position) -> Mode {
        let pairs = self.dig(pos);
        // let rules = pairs.iter().map(|p| p.as_rule());
        let rules = pairs.iter().map(|p| p.as_rule()).collect_vec();
        debug!("rules: {:?}", rules);
        for rule in rules {
            match rule {
                Rule::vertical_mode => return Mode::Vertical,
                Rule::horizontal_mode => return Mode::Horizontal,
                Rule::math_mode => return Mode::Math,
                Rule::headers | Rule::header_stage => return Mode::Header,
                Rule::COMMENT => return Mode::Comment,
                Rule::string_interior => return Mode::Literal,
                _ => continue,
            }
        }
        Mode::Program
    }
}

impl<'a> Search<'a> for DocumentTree<'a> {
    fn pickup(&self, rule: Rule) -> Vec<Pair<'a>> {
        match &self.tree {
            Ok(pair) => pair.pickup(rule),
            Err(_) => vec![],
        }
    }

    fn choose(&self, pos: &Position) -> Option<Pair<'a>> {
        self.tree.as_ref().ok().and_then(|pair| pair.choose(pos))
    }
}

impl<'a> Search<'a> for Pair<'a> {
    fn pickup(&self, rule: Rule) -> Vec<Pair<'a>> {
        let mut vec = vec![];
        let pairs = self.clone().into_inner();
        for pair in pairs {
            if pair.as_rule() == rule {
                vec.push(pair.clone());
            }
            let mut v = pair.pickup(rule);
            vec.append(&mut v);
        }
        vec
    }

    fn choose(&self, pos: &Position) -> Option<Pair<'a>> {
        let pairs = self.clone().into_inner();
        for pair in pairs {
            let range = span_to_range(&pair.as_span());
            if relation(&range, pos) == Relation::In {
                return Some(pair);
            }
        }
        None
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Relation {
    /// 区間が点よりも左にある。
    Left,
    /// 区間が点を含んでいる。
    In,
    /// 区間が点よりも右にある。
    Right,
}

/// pest::Span を lsp_types::Range に変換する。
pub fn span_to_range(s: &Span<'_>) -> Range {
    let start = s.start_pos().line_col();
    let start = Position {
        line: (start.0 - 1) as u32,
        character: (start.1 - 1) as u32,
    };
    let end = s.end_pos().line_col();
    let end = Position {
        line: (end.0 - 1) as u32,
        character: (end.1 - 1) as u32,
    };
    Range { start, end }
}

/// 与えられた範囲と pos の関係を返す。
/// TODO: range と range の関係に一般化する。
fn relation(range: &Range, point: &Position) -> Relation {
    if point < &range.start {
        Relation::Left
    } else if point > &range.end {
        Relation::Right
    } else {
        Relation::In
    }
}
