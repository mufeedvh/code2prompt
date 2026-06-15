+++
title = "Secret scanning"
description = "The --secret-scan policies, the built-in detection rules, path allowlisting, and the .gnawconfig keys."
weight = 50
+++

Secret scanning inspects each file's content for likely credentials before
output. See the [how-to](/how-to/scan-for-secrets/) for the task-oriented walk
through; this page is the exhaustive surface.

## Flags

| Flag | Values | Effect |
| --- | --- | --- |
| `--secret-scan` | `off`, `warn`, `redact`, `block` | What to do on a finding (default `warn`) |
| `--secret-scan-allow <FRAGMENT>` | path substring, repeatable | Skip files whose path contains the fragment |

## Policies

| Policy | Content | Findings | Exit |
| --- | --- | --- | --- |
| `off` | unchanged | none (no scan) | 0 |
| `warn` | unchanged | reported on stderr | 0 |
| `redact` | secret → `[REDACTED: <rule>]` | reported on stderr | 0 |
| `block` | file dropped | reported in abort message | non-zero if any found |

`redact` runs before token counting, so redaction shrinks the reported token
total. Previews in reports show only the first few characters plus a length —
the full secret is never printed or logged.

## Detection rules

Detection is regex match plus a Shannon-entropy gate per rule (a low-entropy
match like a documentation example is rejected). Built-in rules:

| Rule id | Catches |
| --- | --- |
| `aws-access-key-id` | `AKIA…` access key ids |
| `github-pat` | `ghp_`/`gho_`/`ghu_`/`ghs_`/`ghr_` tokens |
| `slack-token` | `xoxb-`/`xoxa-`/`xoxp-`/`xoxr-`/`xoxs-` tokens |
| `google-api-key` | `AIza…` keys |
| `anthropic-key` | `sk-ant-…` keys |
| `openai-key` | `sk-…` / `sk-proj-…` keys |
| `stripe-secret-key` | `sk_live_…` / `rk_live_…` keys |
| `jwt` | `eyJ…`-shaped three-segment tokens |
| `private-key-block` | PEM `-----BEGIN … PRIVATE KEY-----` blocks |
| `crates-io-token` | `cio…` registry tokens |
| `generic-assigned-secret` | a value assigned to a key/secret/token/password-named field |

An allowlist suppresses known false positives (for example AWS's
`AKIAIOSFODNN7EXAMPLE` documentation key and `EXAMPLE`/`dummy`/`placeholder`
strings), so those won't be reported or redacted.

{% aside(kind="note", title="Prefixed vs unprefixed") %}
Rules with a distinctive prefix (`ghp_`, `AKIA`, `cio`, `sk-ant-`) are reliable.
Secrets with no prefix are only caught when they appear in a recognizable
assignment via `generic-assigned-secret` — a bare unprefixed blob on its own
line will not match, by design, to avoid flooding you with false positives.
{% end %}

## Path allowlist

`--secret-scan-allow` (and the `secret_scan_allow_paths` config key) hold
**substring** fragments, not globs — `tests/` skips any path containing that
segment. When the list is empty, gnaw falls back to a built-in default set:

```text
/tests/   /test/   /fixtures/   /testdata/   /__tests__/   _test.
```

Setting any fragment replaces the defaults entirely — you then own the full
list. Allowlisted files are skipped completely, so a real secret inside one is
not detected.

## .gnawconfig keys

| Key | Type | Default |
| --- | --- | --- |
| `secret_scan` | `"off"` / `"warn"` / `"redact"` / `"block"` | `"warn"` |
| `secret_scan_allow_paths` | array of path-substring strings | built-in test set |

```toml
secret_scan = "redact"
secret_scan_allow_paths = ["tests/", "fixtures/"]
```

Resolution order is **CLI flag → `.gnawconfig` → built-in default**.
