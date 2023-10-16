use std::{collections::BTreeMap, fmt::Write};

use crate::flags::{defs::FLAGS, Flag};

const TEMPLATE: &'static str = include_str!("template.rg.1");

pub(crate) fn generate() -> String {
    let mut cats = BTreeMap::new();
    for flag in FLAGS.iter().copied() {
        let mut cat = cats.entry(flag.doc_category()).or_insert(String::new());
        if !cat.is_empty() {
            writeln!(cat, ".sp");
        }
        generate_flag(flag, &mut cat);
    }

    let version = option_env!("CARGO_PKG_VERSION").unwrap_or("N/A");
    let mut out = TEMPLATE.replace("!!VERSION!!", version);
    for (name, value) in cats.iter() {
        let var = format!("!!{name}!!");
        out = out.replace(&var, value);
    }
    out
}

fn generate_flag(flag: &'static dyn Flag, out: &mut String) {
    if let Some(byte) = flag.name_short() {
        let name = char::from(byte);
        write!(out, r"\fB\-{name}\fP");
        if let Some(var) = flag.doc_variable() {
            write!(out, r" \fI{var}\fP");
        }
        write!(out, r", ");
    }

    let name = flag.name_long();
    write!(out, r"\fB\-\-{name}\fP");
    if let Some(var) = flag.doc_variable() {
        write!(out, r"=\fI{var}\fP");
    }
    write!(out, "\n");

    writeln!(out, ".RS 4");
    let doc = flag.doc_long().trim();
    // Convert \flag{foo} into something nicer.
    let doc = super::render_custom_markup(doc, "flag", |name, out| {
        let Some(flag) = crate::flags::parse::lookup(name) else {
            // TODO: Change this to a panic once everything is wired up.
            // unreachable!(r"found unrecognized \flag{{{name}}} in roff docs")
            write!(out, r"\fB\-\-{name}\fP");
            return;
        };
        out.push_str(r"\fB");
        if let Some(name) = flag.name_short() {
            write!(out, r"\-{}/", char::from(name));
        }
        write!(out, r"\-\-{}", flag.name_long());
        out.push_str(r"\fP");
    });
    // Convert \flag-negate{foo} into something nicer.
    let doc = super::render_custom_markup(&doc, "flag-negate", |name, out| {
        let Some(flag) = crate::flags::parse::lookup(name) else {
            // TODO: Change this to a panic once everything is wired up.
            // unreachable!(r"found unrecognized \flag-negate{{{name}}} in roff docs")
            write!(out, r"\fB\-\-{name}\fP");
            return;
        };
        let Some(name) = flag.name_negated() else {
            let long = flag.name_long();
            unreachable!(
                "found \\flag-negate{{{long}}} in roff docs but \
                 {long} does not have a negation"
            );
        };
        out.push_str(r"\fB");
        write!(out, r"\-\-{name}");
        out.push_str(r"\fP");
    });
    writeln!(out, "{doc}");
    if let Some(negated) = flag.name_negated() {
        // Flags that can be negated that aren't switches, like
        // --context-separator, are somewhat weird. Because of that, the docs
        // for those flags should discuss the semantics of negation explicitly.
        // But for switches, the behavior is always the same.
        if flag.is_switch() {
            writeln!(out, ".sp");
            writeln!(
                out,
                r"This flag can be disabled with \fB\-\-{negated}\fP."
            );
        }
    }
    writeln!(out, ".RE");
}

fn flag_long_doc(flag: &'static dyn Flag) -> String {
    let flag_prefix = r"\flag{";
    let original = flag.doc_long().trim();
    let mut out = String::new();
    let mut last_match = 0;
    for (offset, _) in original.match_indices(flag_prefix) {
        out.push_str(&original[last_match..offset]);

        let start = offset + flag_prefix.len();
        let end = start
            + original[start..]
                .find('}')
                .expect(r"found \flag{ without closing }");
        let name = &original[start..end];
        last_match = end + 1;

        let Some(flag) = crate::flags::parse::lookup(name) else {
            // TODO: Change this to a panic once everything is wired up.
            write!(out, r"\fB--{name}\fP");
            continue;
            // unreachable!(r"found unrecognized \flag{{{name}}} in roff docs")
        };
        out.push_str(r"\fB");
        if let Some(name) = flag.name_short() {
            write!(out, r"\-{}/", char::from(name));
        }
        write!(out, r"\-\-{}", flag.name_long());
        out.push_str(r"\fP");
    }
    out.push_str(&original[last_match..]);
    out
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn todo_remove() {
        let dir = PathBuf::from("/tmp/rg-test");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("rg.1"), generate().as_bytes()).unwrap();
    }
}
