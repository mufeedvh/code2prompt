//! Syntax-aware source compression: strip implementation bodies and test
//! modules while preserving signatures, types, doc comments, and structure.
//!
//! Applied per-file *before* token counting, so reported counts reflect the
//! compressed output. Pure text transform; no cross-file state.

use crate::configuration::CompressionOptions;
use std::cmp::Reverse;
use tree_sitter::{Language, Node, Parser, Query, QueryCursor, StreamingIterator};

const BODY_PLACEHOLDER: &str = "{ /* ... */ }";
const TEST_MOD_PLACEHOLDER: &str = "// [test module stripped]";

/// One text replacement over a byte range of the original source.
struct Edit {
    start: usize,
    end: usize,
    replacement: &'static str,
}

pub trait Compressor: Send + Sync {
    fn compress(&self, source: &str, opts: &CompressionOptions) -> String;
}

pub fn compressor_for_extension(ext: &str) -> Option<Box<dyn Compressor>> {
    match ext {
        "rs" => Some(Box::new(RustCompressor)),
        // future: js/ts/py grammars
        _ => None,
    }
}

pub struct RustCompressor;

impl Compressor for RustCompressor {
    fn compress(&self, source: &str, opts: &CompressionOptions) -> String {
        let language: Language = tree_sitter_rust::LANGUAGE.into();
        let mut parser = Parser::new();
        // ABI mismatch or parse failure: fail safe, return source untouched.
        if parser.set_language(&language).is_err() {
            return source.to_string();
        }
        let Some(tree) = parser.parse(source, None) else {
            return source.to_string();
        };
        let root = tree.root_node();

        let mut edits: Vec<Edit> = Vec::new();
        if opts.strip_test_modules {
            edits.extend(test_module_edits(&language, root, source));
        }
        if opts.strip_fn_bodies {
            edits.extend(fn_body_edits(&language, root, source));
        } else if opts.strip_private_bodies {
            // Only meaningful when fn_bodies is off; otherwise all bodies go anyway.
            edits.extend(private_fn_body_edits(&language, root, source));
        }
        if opts.strip_doc_comments {
            edits.extend(doc_comment_edits(&language, root, source));
        }
        if edits.is_empty() {
            return source.to_string();
        }
        splice(source, edits)
    }
}

/// Strip bodies of functions WITHOUT a visibility modifier, keeping signature
/// and return type. Public (pub / pub(crate) / pub(super) / pub(in …)) keep
/// their bodies. Visibility is judged syntactically, on the method itself.
fn private_fn_body_edits(lang: &Language, root: Node, source: &str) -> Vec<Edit> {
    let Ok(query) = Query::new(lang, "(function_item body: (block) @body) @fn") else {
        return Vec::new();
    };
    let (Some(body_cap), Some(fn_cap)) = (
        query.capture_index_for_name("body"),
        query.capture_index_for_name("fn"),
    ) else {
        return Vec::new();
    };
    let mut cursor = QueryCursor::new();
    let mut edits = Vec::new();
    let mut it = cursor.matches(&query, root, source.as_bytes());
    while let Some(m) = it.next() {
        let fn_node = m
            .captures
            .iter()
            .find(|c| c.index == fn_cap)
            .map(|c| c.node);
        let body = m
            .captures
            .iter()
            .find(|c| c.index == body_cap)
            .map(|c| c.node);
        if let (Some(fn_node), Some(body)) = (fn_node, body)
            && !is_public_fn(fn_node)
        {
            edits.push(Edit {
                start: body.start_byte(),
                end: body.end_byte(),
                replacement: BODY_PLACEHOLDER,
            });
        }
    }
    edits
}

/// Every pub-family modifier (`pub`, `pub(crate)`, `pub(super)`, `pub(in …)`)
/// parses as one `visibility_modifier` child, so presence == public here.
fn is_public_fn(fn_node: Node) -> bool {
    let mut c = fn_node.walk();
    fn_node
        .children(&mut c)
        .any(|child| child.kind() == "visibility_modifier")
}

/// Remove doc comments (`///`, `//!`, `/** */`, `/*! */`), keeping regular
/// comments. Classified by byte prefix so it doesn't depend on a grammar
/// version exposing a dedicated doc_comment node.
fn doc_comment_edits(lang: &Language, root: Node, source: &str) -> Vec<Edit> {
    let Ok(query) = Query::new(lang, "[(line_comment) (block_comment)] @c") else {
        return Vec::new();
    };
    let Some(cap) = query.capture_index_for_name("c") else {
        return Vec::new();
    };
    let mut cursor = QueryCursor::new();
    let mut edits = Vec::new();
    let mut it = cursor.matches(&query, root, source.as_bytes());
    while let Some(m) = it.next() {
        for c in m.captures {
            if c.index != cap {
                continue;
            }
            let text = source
                .get(c.node.start_byte()..c.node.end_byte())
                .unwrap_or("");
            if is_doc_comment(text) {
                edits.push(Edit {
                    start: c.node.start_byte(),
                    end: c.node.end_byte(),
                    replacement: "",
                });
            }
        }
    }
    edits
}

fn is_doc_comment(text: &str) -> bool {
    if let Some(rest) = text.strip_prefix("///") {
        return !rest.starts_with('/'); // exclude "////…" (regular)
    }
    if text.starts_with("//!") {
        return true;
    }
    if let Some(rest) = text.strip_prefix("/**") {
        return !rest.starts_with('*') && rest != "/"; // exclude "/***…" and empty "/**/"
    }
    text.starts_with("/*!")
}

/// Capture function/method body blocks. Nested bodies are captured too, but the
/// containment pass in `splice` drops any contained by an outer edit.
fn fn_body_edits(lang: &Language, root: Node, source: &str) -> Vec<Edit> {
    query_edits(
        lang,
        root,
        source,
        "(function_item body: (block) @body)",
        "body",
        BODY_PLACEHOLDER,
    )
}

/// Strip whole `#[cfg(test)] mod ... { ... }` blocks, including the attribute.
fn test_module_edits(lang: &Language, root: Node, source: &str) -> Vec<Edit> {
    let Ok(query) = Query::new(lang, "(mod_item) @m") else {
        return Vec::new();
    };
    let Some(cap) = query.capture_index_for_name("m") else {
        return Vec::new();
    };
    let mut cursor = QueryCursor::new();
    let mut edits = Vec::new();
    let mut it = cursor.matches(&query, root, source.as_bytes());
    while let Some(m) = it.next() {
        for c in m.captures {
            if c.index != cap {
                continue;
            }
            if let Some(start) = test_mod_start(c.node, source) {
                edits.push(Edit {
                    start,
                    end: c.node.end_byte(),
                    replacement: TEST_MOD_PLACEHOLDER,
                });
            }
        }
    }
    edits
}

/// If this module carries a `#[cfg(test)]` attribute, return the start byte of
/// the strip range (the attribute, so it goes too). Otherwise None.
fn test_mod_start(mod_node: Node, source: &str) -> Option<usize> {
    let mut start = None;
    let mut prev = mod_node.prev_sibling();
    while let Some(sib) = prev {
        if sib.kind() != "attribute_item" {
            break; // only walk a contiguous run of attributes
        }
        let text = source.get(sib.start_byte()..sib.end_byte()).unwrap_or("");
        if text.contains("cfg(test)") {
            start = Some(sib.start_byte());
        }
        prev = sib.prev_sibling();
    }
    start
}

fn query_edits(
    lang: &Language,
    root: Node,
    source: &str,
    query_src: &str,
    capture_name: &str,
    replacement: &'static str,
) -> Vec<Edit> {
    let Ok(query) = Query::new(lang, query_src) else {
        return Vec::new();
    };
    let Some(cap) = query.capture_index_for_name(capture_name) else {
        return Vec::new();
    };
    let mut cursor = QueryCursor::new();
    let mut edits = Vec::new();
    let mut it = cursor.matches(&query, root, source.as_bytes());
    while let Some(m) = it.next() {
        for c in m.captures {
            if c.index == cap {
                edits.push(Edit {
                    start: c.node.start_byte(),
                    end: c.node.end_byte(),
                    replacement,
                });
            }
        }
    }
    edits
}

/// Apply edits, dropping any range fully contained in another (e.g. a fn body
/// inside a stripped test module), then splicing right-to-left so earlier byte
/// offsets stay valid as later ones are replaced.
fn splice(source: &str, mut edits: Vec<Edit>) -> String {
    // Outermost-first: smaller start wins; on tie, larger end wins.
    edits.sort_by(|a, b| a.start.cmp(&b.start).then(b.end.cmp(&a.end)));
    let mut keep: Vec<Edit> = Vec::with_capacity(edits.len());
    for e in edits {
        match keep.last() {
            Some(last) if e.start >= last.start && e.end <= last.end => continue, // contained
            _ => keep.push(e),
        }
    }
    // Right-to-left application.
    keep.sort_by_key(|e| Reverse(e.start));
    let mut out = source.to_string();
    for e in keep {
        out.replace_range(e.start..e.end, e.replacement);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"/// Adds two numbers.
pub fn add(a: i32, b: i32) -> i32 {
    let s = a + b;
    s
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_adds() { assert_eq!(2 + 2, 4); }
}
"#;

    #[test]
    fn parser_links() {
        // ABI smoke test: a version mismatch panics here, isolating it from logic.
        let lang: Language = tree_sitter_rust::LANGUAGE.into();
        let mut p = Parser::new();
        p.set_language(&lang)
            .expect("tree-sitter / tree-sitter-rust ABI mismatch");
        assert!(p.parse("fn x() {}", None).is_some());
    }

    fn opts(test: bool, body: bool) -> CompressionOptions {
        CompressionOptions {
            strip_test_modules: test,
            strip_fn_bodies: body,
            ..Default::default()
        }
    }

    #[test]
    fn strips_body_keeps_signature_and_doc() {
        let out = RustCompressor.compress(SAMPLE, &opts(false, true));
        assert!(out.contains("pub fn add(a: i32, b: i32) -> i32"));
        assert!(out.contains("/// Adds two numbers."));
        assert!(!out.contains("let s = a + b;"));
        assert!(out.contains(BODY_PLACEHOLDER));
    }

    #[test]
    fn strips_test_module() {
        let out = RustCompressor.compress(SAMPLE, &opts(true, false));
        assert!(!out.contains("fn it_adds"));
        assert!(!out.contains("cfg(test)"));
        assert!(out.contains("pub fn add")); // non-test code survives
    }

    #[test]
    fn idempotent() {
        let o = opts(true, true);
        let once = RustCompressor.compress(SAMPLE, &o);
        assert_eq!(once, RustCompressor.compress(&once, &o));
    }

    #[test]
    fn shrinks() {
        let out = RustCompressor.compress(SAMPLE, &opts(true, true));
        assert!(out.len() < SAMPLE.len());
    }

    #[test]
    fn private_bodies_strips_private_keeps_public() {
        let src = "pub fn keep() -> i32 { 1 + 1 }\nfn hide() -> i32 { 2 + 2 }\n";
        let o = CompressionOptions {
            strip_private_bodies: true,
            ..Default::default()
        };
        let out = RustCompressor.compress(src, &o);
        assert!(out.contains("1 + 1")); // public body kept
        assert!(out.contains("fn hide() -> i32")); // private signature kept
        assert!(!out.contains("2 + 2")); // private body gone
    }

    #[test]
    fn doc_comments_removed_regular_kept() {
        let src = "/// doc line\n// regular\npub fn f() {}\n";
        let o = CompressionOptions {
            strip_doc_comments: true,
            ..Default::default()
        };
        let out = RustCompressor.compress(src, &o);
        assert!(!out.contains("doc line"));
        assert!(out.contains("// regular"));
    }
}
