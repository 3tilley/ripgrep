use std::{collections::BTreeMap, fmt::Write};

use crate::flags::{defs::FLAGS, Flag};

const TEMPLATE_SHORT: &'static str = include_str!("template.short.help");
const TEMPLATE_LONG: &'static str = include_str!("template.long.help");

pub(crate) fn generate_short() -> String {
    let version = option_env!("CARGO_PKG_VERSION").unwrap_or("N/A");
    let out = TEMPLATE_SHORT.replace("!!VERSION!!", version);
    out
}

pub(crate) fn generate_long() -> String {
    let mut cats = BTreeMap::new();
    for flag in FLAGS.iter().copied() {
        let mut cat = cats.entry(flag.doc_category()).or_insert(String::new());
        if !cat.is_empty() {
            write!(cat, "\n\n");
        }
        generate_long_flag(flag, &mut cat);
    }

    let version = option_env!("CARGO_PKG_VERSION").unwrap_or("N/A");
    let mut out = TEMPLATE_LONG.replace("!!VERSION!!", version);
    for (name, value) in cats.iter() {
        let var = format!("!!{name}!!");
        out = out.replace(&var, value);
    }
    out
}

fn generate_long_flag(flag: &dyn Flag, out: &mut String) {
    if let Some(byte) = flag.name_short() {
        let name = char::from(byte);
        write!(out, r"    -{name}");
        if let Some(var) = flag.doc_variable() {
            write!(out, r" {var}");
        }
        write!(out, r", ");
    } else {
        write!(out, r"    ");
    }

    let name = flag.name_long();
    write!(out, r"--{name}");
    if let Some(var) = flag.doc_variable() {
        write!(out, r"={var}");
    }
    write!(out, "\n");

    let doc = flag.doc_long().trim();
    let doc = super::render_custom_markup(doc, "flag", |name, out| {
        let Some(flag) = crate::flags::parse::lookup(name) else {
            // TODO: Change this to a panic once everything is wired up.
            // unreachable!(r"found unrecognized \flag{{{name}}} in --help docs")
            write!(out, r"--{name}");
            return;
        };
        if let Some(name) = flag.name_short() {
            write!(out, r"-{}/", char::from(name));
        }
        write!(out, r"--{}", flag.name_long());
    });
    let doc = super::render_custom_markup(&doc, "flag-negate", |name, out| {
        let Some(flag) = crate::flags::parse::lookup(name) else {
            // TODO: Change this to a panic once everything is wired up.
            // unreachable!(r"found unrecognized \flag-negate{{{name}}} in --help docs")
            write!(out, r"--{name}");
            return;
        };
        let Some(name) = flag.name_negated() else {
            let long = flag.name_long();
            unreachable!(
                "found \\flag-negate{{{long}}} in --help docs but \
                 {long} does not have a negation"
            );
        };
        write!(out, r"--{name}");
    });

    let mut cleaned = remove_roff(&doc);
    if let Some(negated) = flag.name_negated() {
        write!(cleaned, "\n\nThis flag can be disabled with --{negated}.");
    }
    let indent = " ".repeat(8);
    let wrapopts = textwrap::Options::new(71)
        // Normally I'd be fine with breaking at hyphens, but ripgrep's docs
        // includes a lot of flag names, and they in turn contain hyphens.
        // Breaking flag names across lines is not great.
        .word_splitter(textwrap::WordSplitter::NoHyphenation);
    for (i, paragraph) in cleaned.split("\n\n").enumerate() {
        if i > 0 {
            write!(out, "\n\n");
        }
        let mut new = paragraph.to_string();
        if paragraph.lines().all(|line| line.starts_with("    ")) {
            new = textwrap::indent(&new, &indent).replace(r"\\", r"\");
        } else {
            new = new.replace("\n", " ");
            new = textwrap::refill(&new, &wrapopts);
            new = textwrap::indent(&new, &indent);
        }
        write!(out, "{}", new.trim_end());
    }
}

/// Removes roff syntax from 'v' such that the result is approximately plain
/// text readable.
///
/// This is basically a mish mash of heuristics based on the specific roff used
/// in the docs for the flags in this tool. If new kinds of roff are used in
/// the docs, then this may need to be updated to handle them.
fn remove_roff(v: &str) -> String {
    let mut lines = vec![];
    for line in v.trim().lines() {
        assert!(!line.is_empty(), "roff should have no empty lines");
        if line.starts_with(".") {
            if line.starts_with(".IP ") {
                let item_label = line
                    .split(" ")
                    .nth(1)
                    .expect("first argument to .IP")
                    .replace(r"\(bu", r"•")
                    .replace(r"\fB", "")
                    .replace(r"\fP", ":");
                lines.push(format!("{item_label}"));
            } else if line.starts_with(".IB ") || line.starts_with(".BI ") {
                let pieces = line
                    .split_whitespace()
                    .skip(1)
                    .collect::<Vec<_>>()
                    .concat();
                lines.push(format!("{pieces}"));
            } else if line.starts_with(".sp") || line.starts_with(".PP") {
                lines.push("".to_string());
            }
        } else {
            lines.push(line.to_string());
        }
    }
    lines
        .join("\n")
        .replace(r"\fB", "")
        .replace(r"\fI", "")
        .replace(r"\fP", "")
        .replace(r"\-", "-")
        .replace(r"\\", r"\")
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn todo_remove() {
        let dir = PathBuf::from("/tmp/rg-test");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("rg.short.help"), generate_short().as_bytes())
            .unwrap();
        std::fs::write(dir.join("rg.long.help"), generate_long().as_bytes())
            .unwrap();
    }
}
