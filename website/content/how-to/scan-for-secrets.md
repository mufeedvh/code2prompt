+++
title = "Scan for leaked secrets"
description = "Use --secret-scan to catch API keys, tokens, and private keys before they end up pasted into a chat."
weight = 40
+++

gnaw's output goes straight into a chat box, so a hard-coded API key in your
source is one paste away from leaking. `--secret-scan` checks every file's
content for likely secrets and lets you decide what happens when one is found —
warn, redact it in place, or refuse outright.

```sh
gnaw . --secret-scan redact -O context.md
```

## Choose what happens on a hit

`--secret-scan` takes one of four policies. The default is `warn`.

| Policy | Behavior |
| --- | --- |
| `off` | Don't scan at all |
| `warn` *(default)* | Include content unchanged; report findings on stderr |
| `redact` | Replace each detected secret with `[REDACTED: <rule>]`, then include |
| `block` | Abort the run with a non-zero exit if any secret is found |

```sh
gnaw . --secret-scan warn      # just tell me (default)
gnaw . --secret-scan redact    # scrub it, keep the rest
gnaw . --secret-scan block     # fail the run — good for CI / pre-commit
```

Scanning runs **after** compression and **before** token counting, so when you
redact, the reported token total reflects the scrubbed output, and a secret
buried in a Rust function body that compression already stripped never reaches
the scanner in the first place.

## Reading the report

For `warn` and `redact`, findings print to stderr (your prompt stays clean on
stdout), one line per hit with the rule that matched, location, and a redacted
preview — never the full secret:

```text
[!] secret scan: 1 potential secret(s)
    src/config.rs:12  [github-pat]  ghp_… (40 chars)  (entropy 5.2)
```

For `block`, the same detail appears in the abort message and the process exits
non-zero, which is what makes it useful as a pre-commit or CI gate.

## Skipping test fixtures

Test files often contain deliberately fake tokens that you don't want flagged.
`--secret-scan-allow` takes path fragments (substring match) to skip; it's
repeatable and overrides the config file:

```sh
gnaw . --secret-scan redact --secret-scan-allow tests/ --secret-scan-allow fixtures/
```

If you set nothing, gnaw still skips a built-in set of common test locations
(`/tests/`, `/test/`, `/fixtures/`, `/testdata/`, `/__tests__/`, and `_test.`
filenames). The moment you pass your own fragments, you take full control and
those built-in defaults no longer apply.

{% aside(kind="caution", title="Skipping a path is a real blind spot") %}
Anything matched by the allowlist is **not scanned at all** — a genuine secret
accidentally committed under `tests/` will slide through silently. If you only
want to silence one known-fake fixture token rather than blind a whole
directory, prefer excluding the files from gnaw entirely with `--exclude` so
they're neither scanned nor included.
{% end %}

## Make it the default

Set the policy and allowlist once in `.gnawconfig` (local, or global at
`~/.config/gnaw/.gnawconfig`):

```toml
secret_scan = "redact"
secret_scan_allow_paths = ["tests/", "fixtures/", "examples/"]
```

Precedence is **CLI flag → config file → built-in default (`warn`)**, so a
project can default to `redact` while a one-off run overrides it with
`--secret-scan off`.

## What it catches — and what it can't

gnaw matches known secret shapes with a regex-plus-entropy ruleset (AWS keys,
GitHub tokens, Slack, Google, Anthropic, OpenAI, Stripe, JWTs, PEM private-key
blocks, crates.io tokens, and keyword-anchored assignments). The full list and
how to reason about coverage is in the
[secret-scan reference](/reference/secret-scan/).

{% aside(kind="note", title="Risk reduction, not a guarantee") %}
Pattern scanning reliably catches secrets with a distinctive prefix. It will
miss a custom, unprefixed credential — say a bare 32-character hex key your own
service issues — unless it appears in a recognizable assignment like
`api_key = "…"`. Treat `redact` as a strong safety net, not proof the output is
clean.
{% end %}
