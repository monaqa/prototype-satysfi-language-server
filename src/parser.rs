//! SATySFi のパーサ。

mod peg_parser {
    /// pest parser struct for SATySFi.
    #[derive(Parser)]
    #[grammar = "parser/satysfi.pest"]
    pub struct SatysfiParser;
}

use peg_parser::Rule as SatysfiRule;
use pest::iterators::Pair as PestPair;
use pest::iterators::Pairs as PestPairs;

pub use peg_parser::SatysfiParser;
/// Grammar Rule of SATySFi.
pub type Rule = SatysfiRule;
/// A matching token and everything between them in SATySFi syntax.
pub type Pair<'i> = PestPair<'i, Rule>;
/// An iterator of Pairs.
pub type Pairs<'i> = PestPairs<'i, Rule>;

#[cfg(test)]
mod tests;
