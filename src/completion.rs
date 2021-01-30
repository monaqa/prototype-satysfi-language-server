//! 補完に関する関数群。

use std::collections::HashMap;

use anyhow::anyhow;
use anyhow::Result;
use log::{debug, warn};
use lsp_types::{
    CompletionItem, CompletionList, CompletionParams, CompletionResponse, Documentation,
    InsertTextFormat, MarkupContent, Position,
};
use serde::Deserialize;

use crate::{
    parser::{DocumentTree, Mode},
    Environment,
};

/// デフォルトで用意される補完候補。
const COMPLETION_RESOUCES: &str = include_str!("resource/completion.toml");

/// 補完候補を返す。
pub fn get_completion_response(
    text: Option<&str>,
    params: CompletionParams,
) -> Option<CompletionResponse> {
    let text = text?;

    let pos = params.text_document_position.position;

    let completion_list = get_completion_list(text, &pos);
    Some(CompletionResponse::List(completion_list))
}

/// 無条件で返すことのできる補完候補を取得する。
fn get_completion_list(text: &str, pos: &Position) -> CompletionList {
    let mut cmplist = CompletionList::default();

    let doctree = DocumentTree::from_document(text);
    let mode = doctree.mode(pos);
    debug!("current mode: {:?}", mode);

    let env = Environment::new(&doctree);
    debug!("current environment: {:?}", env);

    match load_completion_resources(mode, env, pos) {
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
    env: Environment,
    pos: &Position,
) -> Result<Vec<CompletionItem>> {
    let items = match mode {
        Mode::Program => load_primitive_completion_items()?,
        Mode::Math => env
            .math_cmds
            .iter()
            .map(|s| CompletionItem::new_simple(s.name.clone(), s.name.clone()))
            .collect(),
        Mode::Horizontal => env
            .inline_cmds
            .iter()
            .map(|s| CompletionItem::new_simple(s.name.clone(), s.name.clone()))
            .collect(),
        Mode::Vertical => env
            .block_cmds
            .iter()
            .map(|s| CompletionItem::new_simple(s.name.clone(), s.name.clone()))
            .collect(),
        _ => vec![],
    };
    Ok(items)
}

/// completion_resources を取得する。
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
