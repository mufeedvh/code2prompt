+++
title = "gnaw's Vision"
description = "Discover the vision behind gnaw and how it enhances LLM interactions with code."
weight = 10
+++

{% aside(kind="note", title="Purpose 🎯") %}
`gnaw` was created to help developers and AI agents interact with codebases
more effectively.
{% end %}

## The Problem 🚩

Large Language Models (LLMs) have revolutionized the way we interact with code.
However, they still face significant challenges with code generation:

- **Planning and Reasoning**: LLMs lack the ability to plan and reason, which is crucial for tasks like code generation, refactoring, and debugging. They often struggle to get the big picture and are short sighted.
- **Context size**: LLMs have a limited context window, which restricts their ability to analyze and understand large codebases.
- **Hallucination**: LLMs can generate code that appears correct but is actually incorrect or nonsensical. This phenomenon, known as hallucination, occurs when the model lacks sufficient context or understanding of the codebase.

This is where `gnaw` comes in.

## The Solution ✅

We believe that planning and reasoning can be achieved by human or AI agents
with scaffolding techniques. These agents need to gather a **high quality
context** of the codebase that is filtered, structured, and formatted for the
task at hand.

The thumb rule would be:

{% aside(kind="tip") %}
Provide as little context as possible, but as much as necessary.
{% end %}

This is practically difficult to achieve, especially for large codebases.
However, `gnaw` is a simple tool that can help developers and AI agents ingest
codebases more effectively.

It automates the process of traversing a codebase, filtering files, and
formatting them into structured prompts that LLMs can understand. By doing so,
it helps to mitigate the challenges of planning, reasoning, and hallucination.

## Architecture ⛩️

`gnaw` is designed in a modular way, allowing for easy integration into various
workflows. It can be used as a core library, a command line / terminal
interface (CLI/TUI), through Python bindings, or — in the future — as a REST
service for browser-extension integration.

### Core

`gnaw-core` is a code ingestion library that streamlines the process of
creating LLM prompts for code analysis, generation, and other tasks. It works
by traversing directories, building a tree structure, and gathering information
about each file. The core library can easily be integrated into other
applications.

### CLI / TUI

The `gnaw` command line interface was designed for humans to generate prompts
directly from your codebase. The generated prompt is automatically copied to
your clipboard and can also be saved to an output file. An interactive TUI lets
you toggle files in and out of the context and watch the token budget update
live. Prompt generation is customizable using Handlebars templates.

### Python

The Python bindings expose the core library for AI agents or automation scripts
that want to interact with codebases seamlessly.

### REST

A REST interface (axum) is planned, so that browser extensions and other
clients can request well-structured context over HTTP.
