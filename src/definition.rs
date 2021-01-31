use itertools::Itertools;
use lsp_types::{GotoDefinitionParams, GotoDefinitionResponse, Location, Position};

use crate::parser::Rule;
use crate::{Buffer, Cst};

/// definition リクエストへの response を返す。
pub fn get_definition_response(
    buf: &Buffer,
    params: GotoDefinitionParams,
) -> Option<GotoDefinitionResponse> {
    let pos = params.text_document_position_params.position;
    let uri = params.text_document_position_params.text_document.uri;

    let buf_cst = &buf.buf_cst;
    let cst = buf.buf_cst.cst.as_ref()?;
    let keyword = find_keyword(cst, &pos)?;
    let cmd_name = buf_cst.as_str(keyword);

    let range = match keyword.rule {
        Rule::math_cmd_name => {
            // 同じ名前のコマンドの定義があった場合は最後を取る。
            buf.env.math_cmds.iter().filter(|cmd| cmd.name == cmd_name).last()?.def_range
        },
        Rule::inline_cmd_name => {
            buf.env.inline_cmds.iter().filter(|cmd| cmd.name == cmd_name).last()?.def_range
        },
        Rule::block_cmd_name => {
            buf.env.block_cmds.iter().filter(|cmd| cmd.name == cmd_name).last()?.def_range
        },
        _ => unreachable!()
    };
    Some(GotoDefinitionResponse::Scalar(Location { uri, range }))
}

/// 与えられたキーワードを見つける。
/// 今の所、キーワードはコマンドのみ。
fn find_keyword<'a>(cst: &'a Cst, pos: &Position) -> Option<&'a Cst> {
    let keywords = cst.dig(&pos);

    for cst in keywords {
        if let Rule::math_cmd_name | Rule::inline_cmd_name | Rule::block_cmd_name = cst.rule {
            return Some(cst);
        }
    }

    None
}
