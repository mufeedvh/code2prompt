+++
title = "Example Guide"
weight = 1
description = "A starter guide page."
+++

Guides lead a user through a specific task they want to accomplish, often with
a sequence of steps.

## Inline cards

Cards and grids also work inline on doc pages via shortcodes:

{% cardgrid(stagger=true) %}
{% card(title="Syntax aware", icon="document") %}
gnaw chunks on whole functions and types, not arbitrary line cuts.
{% end %}
{% card(title="Budgeted", icon="setting") %}
Output is packed to a configurable token budget per target model.
{% end %}
{% end %}

## A code sample

```rust
fn main() {
    let budget = 8_000;
    println!("packing context into {budget} tokens");
}
```
