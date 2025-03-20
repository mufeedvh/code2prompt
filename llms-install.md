# Code2Prompt MCP Server Installation Guide

This guide is specifically designed for AI agents like Cline to install and configure the Repomix MCP server for use with LLM applications like Claude Desktop, Cursor, Roo Code, and Cline.

## Overview of code2prompt-mcp

An MCP server that generates contextual prompts from codebases, making it easier for AI assistants to understand and work with your code repositories.

code2prompt-mcp leverages the high-performance [code2prompt-rs](https://github.com/yourusername/code2prompt-rs) Rust library to analyze codebases and produce structured summaries. It helps bridge the gap between your code and language models by extracting relevant context in a format that's optimized for AI consumption.

## Prerequisites

Before installation, you need:

1. Install rye for dependency management. `curl -sSf https://rye.astral.sh/get | bash` on linux or macOS. Make sure to select to add rye to your PATH when prompted.


## Installation and Configuration

Clone the repository and install dependencies:

```bash
git clone https://github.com/odancona/code2prompt-mcp.git
cd code2prompt-mcp
```

Install all the required dependencies specified in the `pyproject.toml` file in the `.venv` directory with :

```bash
rye build
```

This will create a virtual environment and install all necessary packages.

Then, configure the MCP server configuration file. To run the environnment, you have several options. The first one would be to activate the virtual environment and run the server:

```bash
cd <installation_directory>
source .venv/bin/activate
python code2prompt_mcp.main
```

Alternatively, you can run the server directly using rye:

```bash
rye run python code2prompt_mcp.main
```

It's important to run this command in the cloned directory to use `pyproject.toml` and the virtual environment created by rye.

If you want to be able to run the MCP server from anywhere, you can create a configuration file for your LLM application. Here's an example configuration:

```json
{
  "mcpServers": {
    "code2prompt": {
      "command": "bash",
      "args": [
        "-c",
        "cd /path/to/code2prompt-mcp && rye run python /path/to/code2prompt-mcp/src/code2prompt_mcp/main.py"
      ],
      "env": {}
    }
  }
}
```

## Verify Installation

To verify the installation is working:

1. Restart your LLM application (Cline, Claude Desktop, etc.)
2. Test the connection by running a simple command like:
   ```
   Please get context from /path/to/project for AI analysis using Code2Prompt.
   ```


## Usage Examples

Here are some examples of how to use Code2Prompt MCP server with AI assistants:

### Local Codebase Analysis

```
Can you analyze the code in my project at /path/to/project? Please use Code2prompt MCP to get the context.
```


### Specific File Types Analysis

```
Please get all python files and remove markdown files and the folder tests, use Code2prompt MCP for context.
```
