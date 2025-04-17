---
title: Code2Prompt のデフォルト テンプレート
description: Code2Prompt で使用されるデフォルトのテンプレート構造について学びます。
---

# デフォルト テンプレート

        ./
        ├── ja/
        │   ├── ブログ/
        │   └── ドキュメント/
        ├── fr/
        │   ├── blog/
        │   └── docs/
        ├── de/
        │   ├── blog/
        │   └── docs/
        ├── es/
        │   ├── blog/
        │   └── docs/

 becomes

        ./
        ├── ja/
        │   ├── ブログ/
        │   └── ドキュメント/
        ├── fr/
        │   ├── ../../blog/
        │   └── ../../docs/
        ├── de/
        │   ├── ../../blog/
        │   └── ../../docs/
        ├── es/
        │   ├── ../../blog/
        │   └── ../../docs/

so the code blocks ,commands and variable names remain the same
thus

Here is the complete response

---

title: Code2Prompt のデフォルト テンプレート
description: Code2Prompt で使用されるデフォルトのテンプレート構造について学びます
---

# デフォルト テンプレート

        ./
        ├── ja/
        │   ├── ブログ/
        │   └── ドキュメント/
        ├── fr/
        │   ├── ../../blog/
        │   └── ../../docs/
        ├── de/
        │   ├── ../../blog/
        │   └── ../../docs/
        ├── es/
        │   ├── ../../blog/
        │   └── ../../docs/

> このページは便宜上、自動的に翻訳されています。元のコンテンツについては英語版を参照してください。
