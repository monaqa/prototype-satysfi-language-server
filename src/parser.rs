//! SATySFi のパーサ。

/// SATySFi の PEG パーサ本体。
#[allow(missing_docs)]
mod satysfi_parser {
    #[derive(Parser)]
    #[grammar = "parser/satysfi.pest"]
    pub struct SatysfiParser;
}

pub mod relation;

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
