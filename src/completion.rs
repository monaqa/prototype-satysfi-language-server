//! 補完に関する関数群。

use std::collections::HashMap;

use anyhow::anyhow;
use anyhow::Result;
use itertools::Itertools;
use log::{debug, warn};
use lsp_types::{
    CompletionItem, CompletionList, CompletionParams, CompletionResponse, Documentation,
    InsertTextFormat, MarkupContent, Position,
};
use serde::Deserialize;

use crate::{parser::Mode, Buffer, Environment};

/// デフォルトで用意される補完候補。
const COMPLETION_RESOUCES: &str = include_str!("resource/completion.toml");

/// 補完候補を返す。
pub fn get_completion_response(
    buf: &Buffer,
    params: CompletionParams,
) -> Option<CompletionResponse> {
    let pos = params.text_document_position.position;
    let trigger_char = &params.context.and_then(|ctx| ctx.trigger_character);

    let completion_list = get_completion_list(buf, &pos, trigger_char);
    Some(CompletionResponse::List(completion_list))
}

/// completion_resources を取得する。
fn get_completion_list(buf: &Buffer, pos: &Position, trigger: &Option<String>) -> CompletionList {
    let mut cmplist = CompletionList::default();

    if buf.buf_cst.cst.is_none() {
        return cmplist;
    }

    let mode = buf.buf_cst.cst.as_ref().unwrap().mode(pos);
    debug!("current mode: {:?}", mode);
    let env = &buf.env;

    match load_completion_resources(mode, env, pos, trigger) {
        Ok(res) => {
            cmplist.items = res;
        }
        Err(err) => warn!("failed to load completion resources: {}", err),
    }

    cmplist
}

/// completion_resources を取得する。
fn load_completion_resources(
    mode: Mode,
    env: &Environment,
    _pos: &Position,
    trigger: &Option<String>,
) -> Result<Vec<CompletionItem>> {
    let items = match mode {
        Mode::Program => {
            if let Some(tr) = trigger {
                match tr.as_str() {
                    "#" => vec![], // TODO: 本当は出すべき補完候補がある
                    "+" => vec![],
                    "\\" => vec![],
                    _ => vec![], // unreachable だが致命的ではないのでpanicしない
                }
            } else {
                let mut vars = env.variables
                    .iter()
                    .map(|s| {
                        CompletionItem::new_simple(s.name.clone(), s.name.clone())
                    })
                .collect_vec();
                let primitives = load_primitive_completion_items()?;
                vars.extend(primitives);
                vars
            }
        }

        Mode::Math => {
            let show_cand = { trigger == &Some("\\".to_owned()) };
            if show_cand {
                env.math_cmds
                    .iter()
                    .map(|s| {
                        let mut item = CompletionItem::new_simple(s.name.clone(), s.name.clone());
                        item.insert_text = Some(s.name.clone()[1..].to_owned());
                        item
                    })
                    .collect()
            } else {
                vec![]
            }
        }

        Mode::Horizontal => {
            let show_cand = { trigger == &Some("\\".to_owned()) };
            if show_cand {
                env.inline_cmds
                    .iter()
                    .map(|s| {
                        let mut item = CompletionItem::new_simple(s.name.clone(), s.name.clone());
                        item.insert_text = Some(s.name.clone()[1..].to_owned());
                        item
                    })
                    .collect()
            } else {
                vec![]
            }
        }

        Mode::Vertical => {
            let show_cand = { trigger == &Some("+".to_owned()) };
            if show_cand {
                env.block_cmds
                    .iter()
                    .map(|s| {
                        let mut item = CompletionItem::new_simple(s.name.clone(), s.name.clone());
                        item.insert_text = Some(s.name.clone()[1..].to_owned());
                        item
                    })
                    .collect()
            } else {
                vec![]
            }
        }

        _ => vec![],
    };
    Ok(items)
}

/// プログラムモードのときに返すことのできる補完候補を取得する。
fn load_primitive_completion_items() -> Result<Vec<CompletionItem>> {
    let resources: HashMap<String, Vec<MyCompletionItem>> = toml::from_str(COMPLETION_RESOUCES)?;
    let items = resources
        .into_iter()
        .filter(|(key, _)| key == "primitive")
        .map(|(_, val)| val)
        .next()
        .ok_or_else(|| anyhow!("No field 'primitive' found in completion.toml."))?;
    let items = items.into_iter().map(CompletionItem::from).collect();
    Ok(items)
}

/// TOML ファイルに記述する completion items.
#[derive(Debug, Deserialize)]
struct MyCompletionItem {
    /// The label of this completion item. By default also the text that is inserted when selecting
    /// this completion.
    label: String,
    /// A human-readable string with additional information about this item, like type or symbol
    /// information.
    detail: Option<String>,
    /// A human-readable string that represents a doc-comment.
    documentation: Option<String>,
    /// A string that should be inserted a document when selecting this completion. When falsy the
    /// label is used.
    insert_text: Option<String>,
    /// The format of the insert text. The format applies to both the insertText property and the
    /// newText property of a provided textEdit.
    insert_text_format: Option<String>,
}

impl From<MyCompletionItem> for CompletionItem {
    fn from(my_item: MyCompletionItem) -> Self {
        let mut item = CompletionItem::default();
        item.label = my_item.label;
        item.detail = my_item.detail;
        item.insert_text = my_item.insert_text;
        item.insert_text_format = if my_item.insert_text_format == Some("snippet".to_owned()) {
            Some(InsertTextFormat::Snippet)
        } else {
            None
        };
        item.documentation = my_item.documentation.map(|s| {
            Documentation::MarkupContent(MarkupContent {
                kind: lsp_types::MarkupKind::Markdown,
                value: s,
            })
        });
        item
    }
}
