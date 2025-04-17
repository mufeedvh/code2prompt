---
title: 在 Code2Prompt 中筛选文件
description: 使用不同筛选方法包含或排除文件的逐步指南。
---


## 用法

从代码库目录生成提示：

```sh
code2prompt path/to/codebase
```

使用自定义 Handlebars 模板文件：

```sh
code2prompt path/to/codebase -t path/to/template.hbs
```

使用 glob 模式筛选文件：

```sh
code2prompt path/to/codebase --include="*.rs,*.toml"
```

使用 glob 模式排除文件：

```sh
code2prompt path/to/codebase --exclude="*.txt,*.md"
```

根据排除模式从源树中排除文件/文件夹：

```sh
code2prompt path/to/codebase --exclude="*.npy,*.wav" --exclude-from-tree
```

显示生成的提示的 token 数量：

```sh
code2prompt path/to/codebase --tokens
```

指定 token 计数器的 tokenizer：

```sh
code2prompt path/to/codebase --tokens --encoding=p50k
```

支持的 tokenizer：`cl100k`、`p50k`、`p50k_edit`、`r50k_bas`。
> [!NOTE]  
> 详见 [Tokenizers](#tokenizers)。

将生成的提示保存到输出文件：

```sh
code2prompt path/to/codebase --output=output.txt
```

以 JSON 格式打印输出：

```sh
code2prompt path/to/codebase --json
```

JSON 输出结构如下：

```json
{
  "prompt": "<Generated Prompt>", 
  "directory_name": "codebase",
  "token_count": 1234,
  "model_info": "ChatGPT models, text-embedding-ada-002",
  "files": []
}
```

生成 Git 提交消息（针对暂存文件）：

```sh
code2prompt path/to/codebase --diff -t templates/write-git-commit.hbs
```

生成拉取请求与分支比较（针对暂存文件）：

```sh
code2prompt path/to/codebase --git-diff-branch 'main, development' --git-log-branch 'main, development' -t templates/write-github-pull-request.hbs
```

在源代码块中添加行号：

```sh
code2prompt path/to/codebase --line-number
```

禁用在 Markdown 代码块中换行代码：

```sh
code2prompt path/to/codebase --no-codeblock
```

- 将代码重写为另一种语言。
- 查找错误/安全漏洞。
- 记录代码。
- 实现新功能。

> 我最初编写此工具用于个人使用，以便利用 Claude 3.0 的 200K 上下文窗口，事实证明它非常有用，因此我决定将其开源！

> 为了您的方便，本页面已自动翻译。请参考英文版本获取原始内容。
