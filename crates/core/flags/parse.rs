use std::ffi::OsString;

use {
    aho_corasick::{AhoCorasick, Anchored, Input, Match, PatternID},
    anyhow::Context,
};

use crate::flags::{defs::FLAGS, Args, Flag, FlagValue};

pub(crate) fn parse(
    rawargs: impl IntoIterator<Item = impl Into<OsString>>,
) -> anyhow::Result<Args> {
    let mut args = Args::default();
    Parser::new().parse(rawargs, &mut args)?;
    Ok(args)
}

/// Return the metadata for the flag of the given name.
pub(super) fn lookup(name: &str) -> Option<&'static dyn Flag> {
    // N.B. Creating a new parser might look expensive, but it only builds
    // the lookup trie exactly once. That is, we get a `&'static Parser` from
    // `Parser::new()`.
    match Parser::new().find_long(name) {
        FlagLookup::Match(FlagMatch { flag, .. }) => Some(flag),
        _ => None,
    }
}

#[derive(Debug)]
struct Parser {
    short: Vec<Option<PatternID>>,
    long: AhoCorasick,
    negated: AhoCorasick,
}

impl Parser {
    fn new() -> &'static Parser {
        use std::sync::OnceLock;

        // Since a parser's state is immutable and completely determined by
        // FLAGS, and since FLAGS is a constant, we can initialize it exactly
        // once.
        static P: OnceLock<Parser> = OnceLock::new();
        P.get_or_init(|| {
            // let start = std::time::Instant::now();
            let mut short = vec![None; 128];
            for (i, flag) in FLAGS.iter().enumerate() {
                let Some(name) = flag.name_short() else { continue };
                let pid = PatternID::new(i).expect("small number of flags");
                short[usize::from(name)] = Some(pid);
            }
            let long = AhoCorasick::builder()
                .kind(Some(aho_corasick::AhoCorasickKind::DFA))
                .match_kind(aho_corasick::MatchKind::LeftmostLongest)
                .start_kind(aho_corasick::StartKind::Anchored)
                .prefilter(false)
                .build(FLAGS.iter().map(|f| f.name_long()))
                .expect("automaton of flag names should never fail");
            let negated = AhoCorasick::builder()
                .kind(Some(aho_corasick::AhoCorasickKind::DFA))
                .match_kind(aho_corasick::MatchKind::LeftmostLongest)
                .start_kind(aho_corasick::StartKind::Anchored)
                .prefilter(false)
                // Not all flags have negations. If one isn't present, we just
                // use the empty string. The empty string will match in the
                // case of an unrecognized flag, but since no valid flags are
                // empty, we can treat such a case as a non-match.
                .build(FLAGS.iter().map(|f| f.name_negated().unwrap_or("")))
                .expect("automaton of flag names should never fail");
            // let dur = std::time::Instant::now().duration_since(start);
            // eprintln!("PARSER INIT TIME: {:?}", dur);
            Parser { short, long, negated }
        })
    }

    fn parse<I, O>(&self, rawargs: I, args: &mut Args) -> anyhow::Result<()>
    where
        I: IntoIterator<Item = O>,
        O: Into<OsString>,
    {
        let mut p = lexopt::Parser::from_args(rawargs);
        while let Some(arg) = p.next().context("invalid CLI arguments")? {
            let lookup = match arg {
                lexopt::Arg::Value(value) => {
                    args.positional.push(value);
                    continue;
                }
                lexopt::Arg::Short(ch) => self.find_short(ch),
                lexopt::Arg::Long(name) => self.find_long(name),
            };
            let mat = match lookup {
                FlagLookup::Match(mat) => mat,
                FlagLookup::UnrecognizedShort(name) => {
                    anyhow::bail!("unrecognized flag -{name}")
                }
                FlagLookup::UnrecognizedLong(name) => {
                    anyhow::bail!("unrecognized flag --{name}")
                }
            };
            let value = if mat.negated {
                // Negated flags are always switches, even if the non-negated
                // flag is not. For example, --context-separator accepts a
                // value, but --no-context-separator does not.
                FlagValue::Switch(false)
            } else if mat.flag.is_switch() {
                FlagValue::Switch(true)
            } else {
                FlagValue::Value(p.value().with_context(|| {
                    format!("missing value for flag {mat}")
                })?)
            };
            mat.flag
                .update(value, args)
                .with_context(|| format!("error parsing flag {mat}"))?;
        }
        Ok(())
    }

    fn find_short(&self, ch: char) -> FlagLookup {
        if !ch.is_ascii() {
            return FlagLookup::UnrecognizedShort(ch);
        }
        let byte = u8::try_from(ch).unwrap();
        let Some(index) = self.short[usize::from(byte)] else {
            return FlagLookup::UnrecognizedShort(ch);
        };
        let flag = FLAGS[index];
        FlagLookup::Match(FlagMatch { flag, name: Err(ch), negated: false })
    }

    fn find_long(&self, name: &str) -> FlagLookup {
        if let Some(mat) = find_full_non_empty(&self.long, name) {
            let flag = FLAGS[mat.pattern()];
            FlagLookup::Match(FlagMatch {
                flag,
                name: Ok(flag.name_long()),
                negated: false,
            })
        } else if let Some(mat) = find_full_non_empty(&self.negated, name) {
            let flag = FLAGS[mat.pattern()];
            FlagLookup::Match(FlagMatch {
                flag,
                name: Ok(flag.name_negated().unwrap()),
                negated: true,
            })
        } else {
            FlagLookup::UnrecognizedLong(name.to_string())
        }
    }
}

#[derive(Debug)]
enum FlagLookup {
    Match(FlagMatch),
    UnrecognizedShort(char),
    UnrecognizedLong(String),
}

#[derive(Debug)]
struct FlagMatch {
    flag: &'static dyn Flag,
    name: Result<&'static str, char>,
    negated: bool,
}

impl std::fmt::Display for FlagMatch {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.name {
            Ok(long) => write!(f, "--{long}"),
            Err(short) => write!(f, "-{short}"),
        }
    }
}

/// Look for a match of `name` in the given Aho-Corasick automaton.
///
/// This only returns a match if the one found has a length equivalent to the
/// length of the name given.
fn find_full_non_empty(ac: &AhoCorasick, name: &str) -> Option<Match> {
    let input = Input::new(name).anchored(Anchored::Yes);
    let m = ac.find(input)?;
    // The match must match the name completely. Consider, for example, if the
    // name given is `colors`. If the `colors` flag didn't appear in the FLAGS
    // list for some reason, then `colors` would match the `color` flag.
    //
    // Additionally, if the given name represents a flag that truly does not
    // exist, then it will still match one of the empty string patterns in the
    // Aho-Corasick automaton. (If a flag doesn't have a negation, we add an
    // empty string to the automaton so that the pattern IDs all line up.)
    // Thus, matching the empty string is equivalent to a non-match.
    if m.range().len() != name.len() {
        return None;
    }
    Some(m)
}
