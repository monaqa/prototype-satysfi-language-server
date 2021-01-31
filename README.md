# Prototype SATySFi Language Server

SATySFi language server のプロトタイプ版。
Language server の習作でもあります。
一切の正常動作を保証しませんが、SATySFi の開発に携わる方にテストしていただけると助かります。

## Installation

[Neovim](https://neovim.io) の [coc.nvim](https://github.com/neoclide/coc.nvim) を用いた場合:

まずは以下で `maquette-satysfi-language-server` バイナリをビルド。

```bash
git clone https://github.com/monaqa/maquette-satysfi-language-server
cd maquette-satysfi-language-server
cargo install --path .
```

続いて `coc-settings.json` に以下の設定を追加。

```json:coc-settings.json
{
    "languageserver": {
        "etude": {
            "command": "maquette-satysfi-language-server",
            "filetypes": ["satysfi"]
        }
    }
}
```

この状態で `satysfi` の filetype を有するファイルを開けば language server が起動するかと思います。
なお、デバッグのため Language server を起動させると同時に working directory に `test.log` というファイルが作成され、
language server のログが書き込まれていきます。

## 機能

まだほとんど何も揃っていません。
現時点では一部の Completion のみ対応。

[実装メモ](./memo.md) に今後実装したいものなどを書いています。
