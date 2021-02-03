//! test module for SATySFi parser.

use anyhow::{anyhow, Context, Result};
use itertools::{EitherOrBoth, Itertools};
use pest::Parser;

use super::{Pair, Rule, SatysfiParser};

#[derive(Debug, Clone)]
struct TestPair {
    rule: Rule,
    text: String,
    inner: Vec<TestPair>,
}

fn pair(rule: Rule, text: &str, inner: &[TestPair]) -> TestPair {
    TestPair {
        rule,
        text: text.to_owned(),
        inner: inner.to_owned(),
    }
}

fn assert_parsed(text: &str, test_pair: TestPair) {
    let rule = test_pair.rule;
    let mut pairs = SatysfiParser::parse(rule, text)
        .context(format!(
            r#"Parse failed. rule: {:?}, string: "{}""#,
            rule, text
        ))
        .unwrap();
    let pair = pairs.next().unwrap();

    assert_equal(pair, &test_pair).unwrap();
}

fn assert_equal<'a>(pair: Pair<'a>, test_pair: &TestPair) -> Result<()> {
    if pair.as_rule() != test_pair.rule {
        return Err(anyhow!(
            "Expected rule {:?}, got {:?}.",
            test_pair.rule,
            pair.as_rule()
        ));
    }
    if pair.as_str() != test_pair.text {
        return Err(anyhow!(
            r#"Expected text "{}", got "{}"."#,
            test_pair.text,
            pair.as_str()
        ));
    }

    for elem in pair.into_inner().zip_longest(&test_pair.inner) {
        match elem {
            EitherOrBoth::Both(actual, expect) => {
                assert_equal(actual, expect)?;
            }
            EitherOrBoth::Left(actual) => return Err(anyhow!("Excessive element: {:?}", actual)),
            EitherOrBoth::Right(expect) => return Err(anyhow!("Lacked element: {:?}", expect)),
        }
    }

    Ok(())
}

mod header {

    use super::*;

    #[test]
    fn test_header_stage() {
        assert_parsed(
            "@stage: 0\n",
            pair(
                Rule::header_stage,
                "@stage: 0\n",
                &[pair(Rule::stage, "0", &[])],
            ),
        );
    }
}

mod statement {

    use super::*;

    #[test]
    fn test_type_statement() {
        assert_parsed(
            "type a = int",
            pair(
                Rule::type_stmt,
                "type a = int",
                &[
                    pair(Rule::type_name, "a", &[pair(Rule::var, "a", &[])]),
                    pair(
                        Rule::type_expr,
                        "int",
                        &[pair(
                            Rule::type_prod,
                            "int",
                            &[pair(
                                Rule::type_unary,
                                "int",
                                &[pair(Rule::type_name, "int", &[pair(Rule::var, "int", &[])])],
                            )],
                        )],
                    ),
                ],
            ),
        )
    }
}
