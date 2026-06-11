+++
title = "Example Guide"
weight = 1
description = "A starter guide page."
+++

{% aside(kind="note", title="Tutorial Overview") %}
Welcome to gnaw. This guide walks through turning a repository into an
LLM-ready prompt, trimmed to a token budget.
{% end %}

## Quick start

{{ linkcard(title="Getting started", href="/guides/example/", description="Install gnaw and produce your first prompt.") }}
{{ linkcard(title="Learn filtering", href="/reference/example/", description="Include and exclude files with glob patterns.") }}
{{ linkcard(title="Learn templating", href="/reference/example/", description="Shape output with Handlebars templates.") }}

## Callout variants

{% aside(kind="tip") %}
Use `--var key=value` to supply template variables inline.
{% end %}

{% aside(kind="caution") %}
Passing `--exclude` merges onto config patterns rather than replacing them.
{% end %}

## Integration points

{% tabs() %}
=== Core
A core Rust library that provides the foundation for code ingestion and prompt generation.
=== CLI
Command line interface, designed for humans. `gnaw my_project --include="*.rs"`
=== REST
An axum-based REST surface for browser-extension integration (planned).
{% end %}

## Terminal example

{% code(title="bash") %}
```bash
mkdir -p my_project/{src,tests}
gnaw my_project
```
{% end %}

## A plain code block

```rust
fn main() {
    let budget = 8_000;
    println!("packing context into {budget} tokens");
}
```
