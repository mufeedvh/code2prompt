---
title: Code2Promptでのファイルフィルタリング
description: 異なるフィルタリング方法を使用してファイルをインクルードまたはエクスクルードするステップバイステップガイド。
---


## 使用方法

コードベースディレクトリからプロンプトを生成する:

```sh
code2prompt path/to/codebase
```

カスタムHandlebarsテンプレートファイルを使用する:

```sh
code2prompt path/to/codebase -t path/to/template.hbs
```

グロブパターンを使用してファイルをフィルタリングする:

```sh
code2prompt path/to/codebase --include="*.rs,*.toml"
```

グロブパターンを使用してファイルを除外する:

```sh
code2prompt path/to/codebase --exclude="*.txt,*.md"
```

ソースツリーから除外パターンに基づいてファイル/フォルダを除外する:

```sh
code2prompt path/to/codebase --exclude="*.npy,*.wav" --exclude-from-tree
```

生成されたプロンプトのトークン数を表示する:

```sh
code2prompt path/to/codebase --tokens
```

トークン数にトークナイザを指定する:

```sh
code2prompt path/to/codebase --tokens --encoding=p50k
```

サポートされているトークナイザ: `cl100k`, `p50k`, `p50k_edit`, `r50k_bas`.
> [!注意]  
> 詳細は[トークナイザ](#tokenizers)を参照してください。

生成されたプロンプトを出力ファイルに保存する:

```sh
code2prompt path/to/codebase --output=output.txt
```

出力をJSONとして印刷する:

```sh
code2prompt path/to/codebase --json
```

JSON出力の構造は以下の通りである:

```json
{
  "prompt": "<生成されたプロンプト>", 
  "directory_name": "codebase",
  "token_count": 1234,
  "model_info": "ChatGPTモデル、text-embedding-ada-002",
  "files": []
}
```

Gitコミットメッセージ（ステージングされたファイルに対して）を生成する:

```sh
code2prompt path/to/codebase --diff -t templates/write-git-commit.hbs
```

Pull Requestをブランチ比較（ステージングされたファイルに対して）で生成する:

```sh
code2prompt path/to/codebase --git-diff-branch 'main, development' --git-log-branch 'main, development' -t templates/write-github-pull-request.hbs
```

ソースコードブロックに行番号を追加する:

```sh
code2prompt path/to/codebase --line-number
```

Markdownコードブロック内のコードのラッピングを無効にする:

```sh
code2prompt path/to/codebase --no-codeblock
```

- コードを別の言語に書き直す。
- バグ/セキュリティ脆弱性を発見する。
- コードを文書化する。
- 新しい機能を実装する。

> 最初にこれは、Claude 3.0の200Kコンテキストウィンドウを利用するために個人使用で書いたものであり、かなり役に立ったのでオープンソース化することにした！

> このページは便宜上、自動的に翻訳されています。元のコンテンツについては英語版を参照してください。
