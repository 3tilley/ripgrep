use std::{ffi::OsString, path::PathBuf};

use anyhow::Context as AnyhowContext;

use crate::flags::{
    Args, BinaryMode, BufferMode, CaseMode, ColorChoice, ContextMode,
    CountMode, EncodingMode, EngineChoice, FileMatchMode, Flag, FlagValue,
    LoggingMode, MmapMode, PatternSource, SortMode, SortModeKind, TypeChoice,
};

#[cfg(test)]
use crate::flags::parse;

pub(super) const FLAGS: &'static [&'static dyn Flag] = &[
    &AfterContext,
    &AutoHybridRegex,
    &BeforeContext,
    &Binary,
    &BlockBuffered,
    &ByteOffset,
    &CaseSensitive,
    &Color,
    &Colors,
    &Column,
    &Context,
    &ContextSeparator,
    &Count,
    &CountMatches,
    &Crlf,
    &Debug,
    &DfaSizeLimit,
    &Encoding,
    &Engine,
    &FieldContextSeparator,
    &FieldMatchSeparator,
    &File,
    &Files,
    &FilesWithMatches,
    &FilesWithoutMatch,
    &FixedStrings,
    &Follow,
    &Glob,
    &GlobCaseInsensitive,
    &Heading,
    &Regexp,
    &Trace,
];

/// -A/--after-context
#[derive(Debug)]
struct AfterContext;

impl Flag for AfterContext {
    fn is_switch(&self) -> bool {
        false
    }
    fn name_short(&self) -> Option<u8> {
        Some(b'A')
    }
    fn name_long(&self) -> &'static str {
        "after-context"
    }
    fn doc_variable(&self) -> Option<&'static str> {
        Some("NUM")
    }
    fn doc_category(&self) -> &'static str {
        "output"
    }
    fn doc_short(&self) -> &'static str {
        "Show NUM lines after each match."
    }
    fn doc_long(&self) -> &'static str {
        r"
Show \fINUM\fP lines after each match.
.sp
This overrides the \flag{passthru} flag and partially overrides the
\flag{context} flag.
"
    }

    fn update(&self, v: FlagValue, args: &mut Args) -> anyhow::Result<()> {
        args.context.set_after(convert::usize(&v.unwrap_value())?);
        Ok(())
    }
}

#[cfg(test)]
#[test]
fn test_after_context() {
    // TODO: Add override tests with --passthru.

    let mkctx = |lines| {
        let mut mode = ContextMode::default();
        mode.set_after(lines);
        mode
    };

    let args = parse(None::<&str>).unwrap();
    assert_eq!(ContextMode::default(), args.context);

    let args = parse(["--after-context", "5"]).unwrap();
    assert_eq!(mkctx(5), args.context);

    let args = parse(["--after-context=5"]).unwrap();
    assert_eq!(mkctx(5), args.context);

    let args = parse(["-A", "5"]).unwrap();
    assert_eq!(mkctx(5), args.context);

    let args = parse(["-A5"]).unwrap();
    assert_eq!(mkctx(5), args.context);

    let args = parse(["-A5", "-A10"]).unwrap();
    assert_eq!(mkctx(10), args.context);

    let args = parse(["-A5", "-A0"]).unwrap();
    assert_eq!(mkctx(0), args.context);

    let n = usize::MAX.to_string();
    let args = parse(["--after-context", n.as_str()]).unwrap();
    assert_eq!(mkctx(usize::MAX), args.context);

    #[cfg(target_pointer_width = "64")]
    {
        let n = (u128::from(u64::MAX) + 1).to_string();
        let result = parse(["--after-context", n.as_str()]);
        assert!(result.is_err(), "{result:?}");
    }
}

/// --auto-hybrid-regex
#[derive(Debug)]
struct AutoHybridRegex;

impl Flag for AutoHybridRegex {
    fn is_switch(&self) -> bool {
        true
    }
    fn name_long(&self) -> &'static str {
        "auto-hybrid-regex"
    }
    fn name_negated(&self) -> Option<&'static str> {
        Some("no-auto-hybrid-regex")
    }
    fn doc_category(&self) -> &'static str {
        "search"
    }
    fn doc_short(&self) -> &'static str {
        "(DEPRECATED) Dynamically use PCRE2 if appropriate."
    }
    fn doc_long(&self) -> &'static str {
        r"
DEPRECATED. Use \flag{engine} instead.
.sp
When this flag is used, ripgrep will dynamically choose between supported regex
engines depending on the features used in a pattern. When ripgrep chooses a
regex engine, it applies that choice for every regex provided to ripgrep (e.g.,
via multiple \flag{regexp} or \flag{file} flags).
.sp
As an example of how this flag might behave, ripgrep will attempt to use
its default finite automata based regex engine whenever the pattern can be
successfully compiled with that regex engine. If PCRE2 is enabled and if the
pattern given could not be compiled with the default regex engine, then PCRE2
will be automatically used for searching. If PCRE2 isn't available, then this
flag has no effect because there is only one regex engine to choose from.
.sp
In the future, ripgrep may adjust its heuristics for how it decides which
regex engine to use. In general, the heuristics will be limited to a static
analysis of the patterns, and not to any specific runtime behavior observed
while searching files.
.sp
The primary downside of using this flag is that it may not always be obvious
which regex engine ripgrep uses, and thus, the match semantics or performance
profile of ripgrep may subtly and unexpectedly change. However, in many cases,
all regex engines will agree on what constitutes a match and it can be nice
to transparently support more advanced regex features like look-around and
backreferences without explicitly needing to enable them.
"
    }

    fn update(&self, v: FlagValue, args: &mut Args) -> anyhow::Result<()> {
        let mode = if v.unwrap_switch() {
            EngineChoice::Auto
        } else {
            EngineChoice::Default
        };
        args.engine = mode;
        Ok(())
    }
}

#[cfg(test)]
#[test]
fn test_auto_hybrid_regex() {
    // TODO: Add more tests here with interaction with --engine and -P/--pcre2.

    let args = parse(None::<&str>).unwrap();
    assert_eq!(EngineChoice::Default, args.engine);

    let args = parse(["--auto-hybrid-regex"]).unwrap();
    assert_eq!(EngineChoice::Auto, args.engine);

    let args =
        parse(["--auto-hybrid-regex", "--no-auto-hybrid-regex"]).unwrap();
    assert_eq!(EngineChoice::Default, args.engine);

    let args =
        parse(["--no-auto-hybrid-regex", "--auto-hybrid-regex"]).unwrap();
    assert_eq!(EngineChoice::Auto, args.engine);
}

/// -B/--before-context
#[derive(Debug)]
struct BeforeContext;

impl Flag for BeforeContext {
    fn is_switch(&self) -> bool {
        false
    }
    fn name_short(&self) -> Option<u8> {
        Some(b'B')
    }
    fn name_long(&self) -> &'static str {
        "before-context"
    }
    fn doc_variable(&self) -> Option<&'static str> {
        Some("NUM")
    }
    fn doc_category(&self) -> &'static str {
        "output"
    }
    fn doc_short(&self) -> &'static str {
        "Show NUM lines before each match."
    }
    fn doc_long(&self) -> &'static str {
        r"
Show \fINUM\fP lines before each match.
.sp
This overrides the \flag{passthru} flag and partially overrides the
\flag{context} flag.
"
    }

    fn update(&self, v: FlagValue, args: &mut Args) -> anyhow::Result<()> {
        args.context.set_before(convert::usize(&v.unwrap_value())?);
        Ok(())
    }
}

#[cfg(test)]
#[test]
fn test_before_context() {
    // TODO: Add override tests with --passthru.

    let mkctx = |lines| {
        let mut mode = ContextMode::default();
        mode.set_before(lines);
        mode
    };

    let args = parse(None::<&str>).unwrap();
    assert_eq!(ContextMode::default(), args.context);

    let args = parse(["--before-context", "5"]).unwrap();
    assert_eq!(mkctx(5), args.context);

    let args = parse(["--before-context=5"]).unwrap();
    assert_eq!(mkctx(5), args.context);

    let args = parse(["-B", "5"]).unwrap();
    assert_eq!(mkctx(5), args.context);

    let args = parse(["-B5"]).unwrap();
    assert_eq!(mkctx(5), args.context);

    let args = parse(["-B5", "-B10"]).unwrap();
    assert_eq!(mkctx(10), args.context);

    let args = parse(["-B5", "-B0"]).unwrap();
    assert_eq!(mkctx(0), args.context);

    let n = usize::MAX.to_string();
    let args = parse(["--before-context", n.as_str()]).unwrap();
    assert_eq!(mkctx(usize::MAX), args.context);

    #[cfg(target_pointer_width = "64")]
    {
        let n = (u128::from(u64::MAX) + 1).to_string();
        let result = parse(["--before-context", n.as_str()]);
        assert!(result.is_err(), "{result:?}");
    }
}

/// --binary
#[derive(Debug)]
struct Binary;

impl Flag for Binary {
    fn is_switch(&self) -> bool {
        true
    }
    fn name_long(&self) -> &'static str {
        "binary"
    }
    fn name_negated(&self) -> Option<&'static str> {
        Some("no-binary")
    }
    fn doc_category(&self) -> &'static str {
        "filter"
    }
    fn doc_short(&self) -> &'static str {
        "Search binary files."
    }
    fn doc_long(&self) -> &'static str {
        r"
Enabling this flag will cause ripgrep to search binary files. By default,
ripgrep attempts to automatically skip binary files in order to improve the
relevance of results and make the search faster.
.sp
Binary files are heuristically detected based on whether they contain a
\fBNUL\fP byte or not. By default (without this flag set), once a \fBNUL\fP
byte is seen, ripgrep will stop searching the file. Usually, \fBNUL\fP bytes
occur in the beginning of most binary files. If a \fBNUL\fP byte occurs after
a match, then ripgrep will not print the match, stop searching that file, and
emit a warning that some matches are being suppressed.
.sp
In contrast, when this flag is provided, ripgrep will continue searching a
file even if a \fBNUL\fP byte is found. In particular, if a \fBNUL\fP byte is
found then ripgrep will continue searching until either a match is found or
the end of the file is reached, whichever comes sooner. If a match is found,
then ripgrep will stop and print a warning saying that the search stopped
prematurely.
.sp
If you want ripgrep to search a file without any special \fBNUL\fP byte
handling at all (and potentially print binary data to stdout), then you should
use the \flag{text} flag.
.sp
The \flag{binary} flag is a flag for controlling ripgrep's automatic filtering
mechanism. As such, it does not need to be used when searching a file
explicitly or when searching stdin. That is, it is only applicable when
recursively searching a directory.
.sp
When the \flag{unrestricted} flag is provided for a third time, then this flag
is automatically enabled.
.sp
This flag overrides the \flag{text} flag.
"
    }

    fn update(&self, v: FlagValue, args: &mut Args) -> anyhow::Result<()> {
        args.binary = if v.unwrap_switch() {
            BinaryMode::SearchAndSuppress
        } else {
            BinaryMode::Auto
        };
        Ok(())
    }
}

#[cfg(test)]
#[test]
fn test_binary() {
    // TODO: Add more tests here with interaction with -a/--text.

    let args = parse(None::<&str>).unwrap();
    assert_eq!(BinaryMode::Auto, args.binary);

    let args = parse(["--binary"]).unwrap();
    assert_eq!(BinaryMode::SearchAndSuppress, args.binary);

    let args = parse(["--binary", "--no-binary"]).unwrap();
    assert_eq!(BinaryMode::Auto, args.binary);

    let args = parse(["--no-binary", "--binary"]).unwrap();
    assert_eq!(BinaryMode::SearchAndSuppress, args.binary);
}

/// --block-buffered
#[derive(Debug)]
struct BlockBuffered;

impl Flag for BlockBuffered {
    fn is_switch(&self) -> bool {
        true
    }
    fn name_long(&self) -> &'static str {
        "block-buffered"
    }
    fn name_negated(&self) -> Option<&'static str> {
        Some("no-block-buffered")
    }
    fn doc_category(&self) -> &'static str {
        "output"
    }
    fn doc_short(&self) -> &'static str {
        "Force block buffering."
    }
    fn doc_long(&self) -> &'static str {
        r"
When enabled, ripgrep will use block buffering. That is, whenever a matching
line is found, it will be written to an in-memory buffer and will not be
written to stdout until the buffer reaches a certain size. This is the default
when ripgrep's stdout is redirected to a pipeline or a file. When ripgrep's
stdout is connected to a terminal, line buffering will be used by default.
Forcing block buffering can be useful when dumping a large amount of contents
to a terminal.
.sp
This overrides the \flag{line-buffered} flag.
"
    }

    fn update(&self, v: FlagValue, args: &mut Args) -> anyhow::Result<()> {
        args.buffer = if v.unwrap_switch() {
            BufferMode::Block
        } else {
            BufferMode::Auto
        };
        Ok(())
    }
}

/// --byte-offset
#[derive(Debug)]
struct ByteOffset;

impl Flag for ByteOffset {
    fn is_switch(&self) -> bool {
        true
    }
    fn name_short(&self) -> Option<u8> {
        Some(b'b')
    }
    fn name_long(&self) -> &'static str {
        "byte-offset"
    }
    fn name_negated(&self) -> Option<&'static str> {
        Some("no-byte-offset")
    }
    fn doc_category(&self) -> &'static str {
        "output"
    }
    fn doc_short(&self) -> &'static str {
        "Print the byte offset for each matching line."
    }
    fn doc_long(&self) -> &'static str {
        r"
Print the 0-based byte offset within the input file before each line of output.
If \flag{only-matching} is specified, print the offset of the matched text
itself.
.sp
If ripgrep does transcoding, then the byte offset is in terms of the result
of transcoding and not the original data. This applies similarly to other
transformations on the data, such as decompression or a \flag{pre} filter.
"
    }

    fn update(&self, v: FlagValue, args: &mut Args) -> anyhow::Result<()> {
        args.byte_offset = v.unwrap_switch();
        Ok(())
    }
}

#[cfg(test)]
#[test]
fn test_byte_offset() {
    let args = parse(None::<&str>).unwrap();
    assert_eq!(false, args.byte_offset);

    let args = parse(["--byte-offset"]).unwrap();
    assert_eq!(true, args.byte_offset);

    let args = parse(["-b"]).unwrap();
    assert_eq!(true, args.byte_offset);

    let args = parse(["--byte-offset", "--no-byte-offset"]).unwrap();
    assert_eq!(false, args.byte_offset);

    let args = parse(["--no-byte-offset", "-b"]).unwrap();
    assert_eq!(true, args.byte_offset);
}

/// -s/--case-sensitive
#[derive(Debug)]
struct CaseSensitive;

impl Flag for CaseSensitive {
    fn is_switch(&self) -> bool {
        true
    }
    fn name_short(&self) -> Option<u8> {
        Some(b's')
    }
    fn name_long(&self) -> &'static str {
        "case-sensitive"
    }
    fn doc_category(&self) -> &'static str {
        "search"
    }
    fn doc_short(&self) -> &'static str {
        r"Search case sensitively (default)."
    }
    fn doc_long(&self) -> &'static str {
        r"
Execute the search case sensitively. This is the default mode.
.sp
This is a global option that applies to all patterns given to ripgrep.
Individual patterns can still be matched case insensitively by using inline
regex flags. For example, \fB(?i)abc\fP will match \fBabc\fP case insensitively
even when this flag is used.
.sp
This flag overrides the \flag{ignore-case} and \flag{smart-case} flags.
"
    }

    fn update(&self, v: FlagValue, args: &mut Args) -> anyhow::Result<()> {
        assert!(v.unwrap_switch(), "flag has no negation");
        args.case = CaseMode::Sensitive;
        Ok(())
    }
}

#[cfg(test)]
#[test]
fn test_case_sensitive() {
    let args = parse(None::<&str>).unwrap();
    assert_eq!(CaseMode::Sensitive, args.case);

    let args = parse(["--case-sensitive"]).unwrap();
    assert_eq!(CaseMode::Sensitive, args.case);

    let args = parse(["-s"]).unwrap();
    assert_eq!(CaseMode::Sensitive, args.case);
}

/// --color
#[derive(Debug)]
struct Color;

impl Flag for Color {
    fn is_switch(&self) -> bool {
        false
    }
    fn name_long(&self) -> &'static str {
        "color"
    }
    fn doc_variable(&self) -> Option<&'static str> {
        Some("WHEN")
    }
    fn doc_category(&self) -> &'static str {
        "output"
    }
    fn doc_short(&self) -> &'static str {
        "When to use color."
    }
    fn doc_long(&self) -> &'static str {
        r"
This flag controls when to use colors. The default setting is \fBauto\fP, which
means ripgrep will try to guess when to use colors. For example, if ripgrep is
printing to a terminal, then it will use colors, but if it is redirected to a
file or a pipe, then it will suppress color output.
.sp
ripgrep will suppress color output by default in some other circumstances as
well. These include, but are not limited to:
.sp
.IP \(bu 3n
When the \fBTERM\fP environment variable is not set or set to \fBdumb\fP.
.sp
.IP \(bu 3n
When the \fBNO_COLOR\fP environment variable is set (regardless of value).
.sp
.IP \(bu 3n
When flags that imply no use for colors are given. For example,
\flag{vimgrep} and \flag{json}.
.
.PP
The possible values for this flag are:
.sp
.IP \fBnever\fP 10n
Colors will never be used.
.sp
.IP \fBauto\fP 10n
The default. ripgrep tries to be smart.
.sp
.IP \fBalways\fP 10n
Colors will always be used regardless of where output is sent.
.sp
.IP \fBansi\fP 10n
Like 'always', but emits ANSI escapes (even in a Windows console).
.
.PP
This flag also controls whether hyperlinks are emitted. For example, when
a hyperlink format is specified, hyperlinks won't be used when color is
suppressed. If one wants to emit hyperlinks but no colors, then one must use
the \flag{colors} flag to manually set all color styles to \fBnone\fP:
.sp
.EX
    \-\-colors 'path:none' \\
    \-\-colors 'line:none' \\
    \-\-colors 'column:none' \\
    \-\-colors 'match:none'
.EE
.sp
"
    }
    fn doc_choices(&self) -> &'static [&'static str] {
        &["never", "auto", "always", "ansi"]
    }

    fn update(&self, v: FlagValue, args: &mut Args) -> anyhow::Result<()> {
        args.color = match convert::str(&v.unwrap_value())? {
            "never" => ColorChoice::Never,
            "auto" => ColorChoice::Auto,
            "always" => ColorChoice::Always,
            "ansi" => ColorChoice::Ansi,
            unk => anyhow::bail!("choice '{unk}' is unrecognized"),
        };
        Ok(())
    }
}

#[cfg(test)]
#[test]
fn test_color() {
    let args = parse(None::<&str>).unwrap();
    assert_eq!(ColorChoice::Auto, args.color);

    let args = parse(["--color", "never"]).unwrap();
    assert_eq!(ColorChoice::Never, args.color);

    let args = parse(["--color", "auto"]).unwrap();
    assert_eq!(ColorChoice::Auto, args.color);

    let args = parse(["--color", "always"]).unwrap();
    assert_eq!(ColorChoice::Always, args.color);

    let args = parse(["--color", "ansi"]).unwrap();
    assert_eq!(ColorChoice::Ansi, args.color);

    let args = parse(["--color=never"]).unwrap();
    assert_eq!(ColorChoice::Never, args.color);

    let args = parse(["--color", "always", "--color", "never"]).unwrap();
    assert_eq!(ColorChoice::Never, args.color);

    let args = parse(["--color", "never", "--color", "always"]).unwrap();
    assert_eq!(ColorChoice::Always, args.color);

    let result = parse(["--color", "foofoo"]);
    assert!(result.is_err(), "{result:?}");

    let result = parse(["--color", "Always"]);
    assert!(result.is_err(), "{result:?}");
}

/// --colors
#[derive(Debug)]
struct Colors;

impl Flag for Colors {
    fn is_switch(&self) -> bool {
        false
    }
    fn name_long(&self) -> &'static str {
        "colors"
    }
    fn doc_variable(&self) -> Option<&'static str> {
        Some("COLOR_SPEC")
    }
    fn doc_category(&self) -> &'static str {
        "output"
    }
    fn doc_short(&self) -> &'static str {
        "Configure color settings and styles."
    }
    fn doc_long(&self) -> &'static str {
        r"
This flag specifies color settings for use in the output. This flag may be
provided multiple times. Settings are applied iteratively. Pre-existing color
labels are limited to one of eight choices: \fBred\fP, \fBblue\fP, \fBgreen\fP,
\fBcyan\fP, \fBmagenta\fP, \fByellow\fP, \fBwhite\fP and \fBblack\fP. Styles
are limited to \fBnobold\fP, \fBbold\fP, \fBnointense\fP, \fBintense\fP,
\fBnounderline\fP or \fBunderline\fP.
.sp
The format of the flag is
\fB{\fP\fItype\fP\fB}:{\fP\fIattribute\fP\fB}:{\fP\fIvalue\fP\fB}\fP.
\fItype\fP should be one of \fBpath\fP, \fBline\fP, \fBcolumn\fP or
\fBmatch\fP. \fIattribute\fP can be \fBfg\fP, \fBbg\fP or \fBstyle\fP.
\fIvalue\fP is either a color (for \fBfg\fP and \fBbg\fP) or a text style. A
special format, \fB{\fP\fItype\fP\fB}:none\fP, will clear all color settings
for \fItype\fP.
.sp
For example, the following command will change the match color to magenta and
the background color for line numbers to yellow:
.sp
.EX
    rg \-\-colors 'match:fg:magenta' \-\-colors 'line:bg:yellow'
.EE
.sp
Extended colors can be used for \fIvalue\fP when the terminal supports
ANSI color sequences. These are specified as either \fIx\fP (256-color) or
.IB x , x , x
(24-bit truecolor) where \fIx\fP is a number between \fB0\fP and \fB255\fP
inclusive. \fIx\fP may be given as a normal decimal number or a hexadecimal
number, which is prefixed by \fB0x\fP.
.sp
For example, the following command will change the match background color to
that represented by the rgb value (0,128,255):
.sp
.EX
    rg \-\-colors 'match:bg:0,128,255'
.EE
.sp
or, equivalently,
.sp
.EX
    rg \-\-colors 'match:bg:0x0,0x80,0xFF'
.EE
.sp
Note that the \fBintense\fP and \fBnointense\fP styles will have no effect when
used alongside these extended color codes.
"
    }

    fn update(&self, v: FlagValue, args: &mut Args) -> anyhow::Result<()> {
        let v = v.unwrap_value();
        let v = convert::str(&v)?;
        args.colors.push(v.parse()?);
        Ok(())
    }
}

#[cfg(test)]
#[test]
fn test_colors() {
    let args = parse(None::<&str>).unwrap();
    assert!(args.colors.is_empty());

    let args = parse(["--colors", "match:fg:magenta"]).unwrap();
    assert_eq!(args.colors, vec!["match:fg:magenta".parse().unwrap()]);

    let args =
        parse(["--colors", "match:fg:magenta", "--colors", "line:bg:yellow"])
            .unwrap();
    assert_eq!(
        args.colors,
        vec![
            "match:fg:magenta".parse().unwrap(),
            "line:bg:yellow".parse().unwrap()
        ]
    );
}

/// --column
#[derive(Debug)]
struct Column;

impl Flag for Column {
    fn is_switch(&self) -> bool {
        true
    }
    fn name_long(&self) -> &'static str {
        "column"
    }
    fn name_negated(&self) -> Option<&'static str> {
        Some("no-column")
    }
    fn doc_category(&self) -> &'static str {
        "output"
    }
    fn doc_short(&self) -> &'static str {
        "Show column numbers."
    }
    fn doc_long(&self) -> &'static str {
        r"
Show column numbers (1-based). This only shows the column numbers for the first
match on each line. This does not try to account for Unicode. One byte is equal
to one column. This implies \flag{line-number}.
.sp
When \flag{only-matching} is used, then the column numbers written correspond
to the start of each match.
"
    }

    fn update(&self, v: FlagValue, args: &mut Args) -> anyhow::Result<()> {
        args.column = v.unwrap_switch();
        Ok(())
    }
}

#[cfg(test)]
#[test]
fn test_column() {
    let args = parse(None::<&str>).unwrap();
    assert_eq!(false, args.column);

    let args = parse(["--column"]).unwrap();
    assert_eq!(true, args.column);

    let args = parse(["--column", "--no-column"]).unwrap();
    assert_eq!(false, args.column);

    let args = parse(["--no-column", "--column"]).unwrap();
    assert_eq!(true, args.column);
}

/// -C/--context
#[derive(Debug)]
struct Context;

impl Flag for Context {
    fn is_switch(&self) -> bool {
        false
    }
    fn name_short(&self) -> Option<u8> {
        Some(b'C')
    }
    fn name_long(&self) -> &'static str {
        "context"
    }
    fn doc_variable(&self) -> Option<&'static str> {
        Some("NUM")
    }
    fn doc_category(&self) -> &'static str {
        "output"
    }
    fn doc_short(&self) -> &'static str {
        r"Show NUM lines before and after each match."
    }
    fn doc_long(&self) -> &'static str {
        r"
Show \fINUM\fP lines before and after each match. This is equivalent to
providing both the \flag{before\-context} and \flag{after-context} flags with
the same value.
.sp
This overrides the \flag{passthru} flag. The \flag{after-context} and
\flag{before-context} flags both partially override this flag, regardless of
the order. For example, \fB\-A2 \-C1\fP is equivalent to \fB\-A2 \-B1\fP.
"
    }

    fn update(&self, v: FlagValue, args: &mut Args) -> anyhow::Result<()> {
        args.context.set_both(convert::usize(&v.unwrap_value())?);
        Ok(())
    }
}

#[cfg(test)]
#[test]
fn test_context() {
    // TODO: Add override tests with --passthru.

    let mkctx = |lines| {
        let mut mode = ContextMode::default();
        mode.set_both(lines);
        mode
    };

    let args = parse(None::<&str>).unwrap();
    assert_eq!(ContextMode::default(), args.context);

    let args = parse(["--context", "5"]).unwrap();
    assert_eq!(mkctx(5), args.context);

    let args = parse(["--context=5"]).unwrap();
    assert_eq!(mkctx(5), args.context);

    let args = parse(["-C", "5"]).unwrap();
    assert_eq!(mkctx(5), args.context);

    let args = parse(["-C5"]).unwrap();
    assert_eq!(mkctx(5), args.context);

    let args = parse(["-C5", "-C10"]).unwrap();
    assert_eq!(mkctx(10), args.context);

    let args = parse(["-C5", "-C0"]).unwrap();
    assert_eq!(mkctx(0), args.context);

    let n = usize::MAX.to_string();
    let args = parse(["--context", n.as_str()]).unwrap();
    assert_eq!(mkctx(usize::MAX), args.context);

    #[cfg(target_pointer_width = "64")]
    {
        let n = (u128::from(u64::MAX) + 1).to_string();
        let result = parse(["--context", n.as_str()]);
        assert!(result.is_err(), "{result:?}");
    }

    // Test the interaction between -A/-B and -C. Basically, -A/-B always
    // partially overrides -C, regardless of where they appear relative to
    // each other. This behavior is also how GNU grep works, and it also makes
    // logical sense to me: -A/-B are the more specific flags.
    let args = parse(["-A1", "-C5"]).unwrap();
    let mut mode = ContextMode::default();
    mode.set_after(1);
    mode.set_both(5);
    assert_eq!(mode, args.context);
    assert_eq!((5, 1), args.context.get_limited());

    let args = parse(["-B1", "-C5"]).unwrap();
    let mut mode = ContextMode::default();
    mode.set_before(1);
    mode.set_both(5);
    assert_eq!(mode, args.context);
    assert_eq!((1, 5), args.context.get_limited());

    let args = parse(["-A1", "-B2", "-C5"]).unwrap();
    let mut mode = ContextMode::default();
    mode.set_before(2);
    mode.set_after(1);
    mode.set_both(5);
    assert_eq!(mode, args.context);
    assert_eq!((2, 1), args.context.get_limited());

    // These next three are like the ones above, but with -C before -A/-B. This
    // tests that -A and -B only partially override -C. That is, -C1 -A2 is
    // equivalent to -B1 -A2.
    let args = parse(["-C5", "-A1"]).unwrap();
    let mut mode = ContextMode::default();
    mode.set_after(1);
    mode.set_both(5);
    assert_eq!(mode, args.context);
    assert_eq!((5, 1), args.context.get_limited());

    let args = parse(["-C5", "-B1"]).unwrap();
    let mut mode = ContextMode::default();
    mode.set_before(1);
    mode.set_both(5);
    assert_eq!(mode, args.context);
    assert_eq!((1, 5), args.context.get_limited());

    let args = parse(["-C5", "-A1", "-B2"]).unwrap();
    let mut mode = ContextMode::default();
    mode.set_before(2);
    mode.set_after(1);
    mode.set_both(5);
    assert_eq!(mode, args.context);
    assert_eq!((2, 1), args.context.get_limited());
}

/// --context-separator
#[derive(Debug)]
struct ContextSeparator;

impl Flag for ContextSeparator {
    fn is_switch(&self) -> bool {
        false
    }
    fn name_long(&self) -> &'static str {
        "context-separator"
    }
    fn name_negated(&self) -> Option<&'static str> {
        Some("no-context-separator")
    }
    fn doc_variable(&self) -> Option<&'static str> {
        Some("SEPARATOR")
    }
    fn doc_category(&self) -> &'static str {
        "output"
    }
    fn doc_short(&self) -> &'static str {
        r"Set the separator used between contextual chunks."
    }
    fn doc_long(&self) -> &'static str {
        r"
The string used to separate non-contiguous context lines in the output. This is
only used when one of the context flags is used (that is, \flag{after-context},
\flag{before-context} or \flag{context}). Escape sequences like \fB\\x7F\fP or
\fB\\t\fP may be used. The default value is \fB\-\-\fP.
.sp
When the context separator is set to an empty string, then a line break
is still inserted. To completely disable context separators, use the
\flag-negate{context-separator} flag.
"
    }

    fn update(&self, v: FlagValue, args: &mut Args) -> anyhow::Result<()> {
        use crate::flags::ContextSeparator as Separator;

        args.context_separator = match v {
            FlagValue::Switch(true) => {
                unreachable!("flag can only be disabled")
            }
            FlagValue::Switch(false) => Separator::disabled(),
            FlagValue::Value(v) => Separator::new(&v)?,
        };
        Ok(())
    }
}

#[cfg(test)]
#[test]
fn test_context_separator() {
    use bstr::BString;

    use crate::flags::ContextSeparator as Separator;

    let getbytes = |ctxsep: Separator| ctxsep.into_bytes().map(BString::from);

    let args = parse(None::<&str>).unwrap();
    assert_eq!(Some(BString::from("--")), getbytes(args.context_separator));

    let args = parse(["--context-separator", "XYZ"]).unwrap();
    assert_eq!(Some(BString::from("XYZ")), getbytes(args.context_separator));

    let args = parse(["--no-context-separator"]).unwrap();
    assert_eq!(None, getbytes(args.context_separator));

    let args = parse(["--context-separator", "XYZ", "--no-context-separator"])
        .unwrap();
    assert_eq!(None, getbytes(args.context_separator));

    let args = parse(["--no-context-separator", "--context-separator", "XYZ"])
        .unwrap();
    assert_eq!(Some(BString::from("XYZ")), getbytes(args.context_separator));

    // This checks that invalid UTF-8 can be used. This case isn't too tricky
    // to handle, because it passes the invalid UTF-8 as an escape sequence
    // that is itself valid UTF-8. It doesn't become invalid UTF-8 until after
    // the argument is parsed and then unescaped.
    let args = parse(["--context-separator", r"\xFF"]).unwrap();
    assert_eq!(Some(BString::from(b"\xFF")), getbytes(args.context_separator));

    // In this case, we specifically try to pass an invalid UTF-8 argument to
    // the flag. In theory we might be able to support this, but because we do
    // unescaping and because unescaping wants valid UTF-8, we do a UTF-8 check
    // on the value. Since we pass invalid UTF-8, it fails. This demonstrates
    // that the only way to use an invalid UTF-8 separator is by specifying an
    // escape sequence that is itself valid UTF-8.
    #[cfg(unix)]
    {
        use std::{ffi::OsStr, os::unix::ffi::OsStrExt};

        let result = parse([
            OsStr::from_bytes(b"--context-separator"),
            OsStr::from_bytes(&[0xFF]),
        ]);
        assert!(result.is_err(), "{result:?}");
    }
}

/// -c/--count
#[derive(Debug)]
struct Count;

impl Flag for Count {
    fn is_switch(&self) -> bool {
        true
    }
    fn name_short(&self) -> Option<u8> {
        Some(b'c')
    }
    fn name_long(&self) -> &'static str {
        "count"
    }
    fn doc_category(&self) -> &'static str {
        "output-modes"
    }
    fn doc_short(&self) -> &'static str {
        r"Only show the count of matching lines for each file."
    }
    fn doc_long(&self) -> &'static str {
        r"
This flag suppresses normal output and shows the number of lines that match the
given patterns for each file searched. Each file containing a match has its
path and count printed on each line. Note that unless \flag{multiline}
is enabled, this reports the number of lines that match and not the total
number of matches. In multiline mode, \flag{count} is equivalent to
\flag{count-matches}.
.sp
If only one file is given to ripgrep, then only the count is printed if there
is a match. The \flag{with-filename} flag can be used to force printing the
file path in this case. If you need a count to be printed regardless of whether
there is a match, then use \flag{include-zero}.
.sp
This overrides the \flag{count-matches} flag. Note that when \flag{count}
is combined with \flag{only-matching}, then ripgrep behaves as if
\flag{count-matches} was given.
"
    }

    fn update(&self, v: FlagValue, args: &mut Args) -> anyhow::Result<()> {
        let yes = v.unwrap_switch();
        assert!(yes, "--count can only be enabled");
        args.count = Some(CountMode::Lines);
        Ok(())
    }
}

#[cfg(test)]
#[test]
fn test_count() {
    let args = parse(None::<&str>).unwrap();
    assert_eq!(None, args.count);

    let args = parse(["--count"]).unwrap();
    assert_eq!(Some(CountMode::Lines), args.count);

    let args = parse(["-c"]).unwrap();
    assert_eq!(Some(CountMode::Lines), args.count);

    let args = parse(["--count-matches", "--count"]).unwrap();
    assert_eq!(Some(CountMode::Lines), args.count);

    let args = parse(["--count-matches", "-c"]).unwrap();
    assert_eq!(Some(CountMode::Lines), args.count);
}

/// --count-matches
#[derive(Debug)]
struct CountMatches;

impl Flag for CountMatches {
    fn is_switch(&self) -> bool {
        true
    }
    fn name_long(&self) -> &'static str {
        "count-matches"
    }
    fn doc_variable(&self) -> Option<&'static str> {
        None
    }
    fn doc_category(&self) -> &'static str {
        "output-modes"
    }
    fn doc_short(&self) -> &'static str {
        r"Only show the count of individual matches for each file."
    }
    fn doc_long(&self) -> &'static str {
        r"
This flag suppresses normal output and shows the number of individual matches
of the given patterns for each file searched. Each file containing matches has
its path and match count printed on each line. Note that this reports the total
number of individual matches and not the number of lines that match.
.sp
If only one file is given to ripgrep, then only the count is printed if there
is a match. The \flag{with-filename} flag can be used to force printing the
file path in this case.
.sp
This overrides the \flag{count} flag. Note that when \flag{count} is combined
with \flag{only-matching}, then ripgrep behaves as if \flag{count-matches} was
given.
"
    }

    fn update(&self, v: FlagValue, args: &mut Args) -> anyhow::Result<()> {
        let yes = v.unwrap_switch();
        assert!(yes, "--count-matches can only be enabled");
        args.count = Some(CountMode::All);
        Ok(())
    }
}

#[cfg(test)]
#[test]
fn test_count_matches() {
    let args = parse(None::<&str>).unwrap();
    assert_eq!(None, args.count);

    let args = parse(["--count-matches"]).unwrap();
    assert_eq!(Some(CountMode::All), args.count);

    let args = parse(["--count", "--count-matches"]).unwrap();
    assert_eq!(Some(CountMode::All), args.count);

    let args = parse(["-c", "--count-matches"]).unwrap();
    assert_eq!(Some(CountMode::All), args.count);
}

/// --crlf
#[derive(Debug)]
struct Crlf;

impl Flag for Crlf {
    fn is_switch(&self) -> bool {
        true
    }
    fn name_long(&self) -> &'static str {
        "crlf"
    }
    fn name_negated(&self) -> Option<&'static str> {
        Some("no-crlf")
    }
    fn doc_category(&self) -> &'static str {
        "search"
    }
    fn doc_short(&self) -> &'static str {
        r"Support CRLF line terminators (useful on Windows)."
    }
    fn doc_long(&self) -> &'static str {
        r"
When enabled, ripgrep will treat CRLF (\fB\\r\\n\fP) as a line terminator
instead of just \fB\\n\fP.
.sp
Principally, this permits the line anchor assertions \fB^\fP and \fB$\fP in
regex patterns to treat CRLF, CR or LF as line terminators instead of just LF.
Note that they will never match between a CR and a LF. CRLF is treated as one
single line terminator.
.sp
When using the default regex engine, CRLF support can also be enabled inside
the pattern with the \fBR\fP flag. For example, \fB(?R:$)\fP will match just
before either CR or LF, but never between CR and LF.
"
    }

    fn update(&self, v: FlagValue, args: &mut Args) -> anyhow::Result<()> {
        args.crlf = v.unwrap_switch();
        if args.crlf {
            args.null_data = false;
        }
        Ok(())
    }
}

#[cfg(test)]
#[test]
fn test_crlf() {
    // TODO: Add more tests with --null-data.

    let args = parse(None::<&str>).unwrap();
    assert_eq!(false, args.crlf);

    let args = parse(["--crlf"]).unwrap();
    assert_eq!(true, args.crlf);
}

/// --debug
#[derive(Debug)]
struct Debug;

impl Flag for Debug {
    fn is_switch(&self) -> bool {
        true
    }
    fn name_long(&self) -> &'static str {
        "debug"
    }
    fn doc_category(&self) -> &'static str {
        "logging"
    }
    fn doc_short(&self) -> &'static str {
        r"Show debug messages."
    }
    fn doc_long(&self) -> &'static str {
        r"
Show debug messages. Please use this when filing a bug report.
.sp
The \flag{debug} flag is generally useful for figuring out why ripgrep skipped
searching a particular file. The debug messages should mention all files
skipped and why they were skipped.
.sp
To get even more debug output, use the \flag{trace} flag, which implies
\flag{debug} along with additional trace data.
"
    }

    fn update(&self, v: FlagValue, args: &mut Args) -> anyhow::Result<()> {
        assert!(v.unwrap_switch(), "--debug can only be enabled");
        args.logging = Some(LoggingMode::Debug);
        Ok(())
    }
}

#[cfg(test)]
#[test]
fn test_debug() {
    let args = parse(None::<&str>).unwrap();
    assert_eq!(None, args.logging);

    let args = parse(["--debug"]).unwrap();
    assert_eq!(Some(LoggingMode::Debug), args.logging);

    let args = parse(["--trace", "--debug"]).unwrap();
    assert_eq!(Some(LoggingMode::Debug), args.logging);
}

/// --dfa-size-limit
#[derive(Debug)]
struct DfaSizeLimit;

impl Flag for DfaSizeLimit {
    fn is_switch(&self) -> bool {
        false
    }
    fn name_long(&self) -> &'static str {
        "dfa-size-limit"
    }
    fn doc_variable(&self) -> Option<&'static str> {
        Some("NUM+SUFFIX?")
    }
    fn doc_category(&self) -> &'static str {
        "search"
    }
    fn doc_short(&self) -> &'static str {
        r"The upper size limit of the regex DFA."
    }
    fn doc_long(&self) -> &'static str {
        r"
The upper size limit of the regex DFA. The default limit is something generous
for any single pattern or for many smallish patterns. This should only be
changed on very large regex inputs where the (slower) fallback regex engine may
otherwise be used if the limit is reached.
.sp
The input format accepts suffixes of \fBK\fP, \fBM\fP or \fBG\fP which
correspond to kilobytes, megabytes and gigabytes, respectively. If no suffix is
provided the input is treated as bytes.
"
    }

    fn update(&self, v: FlagValue, args: &mut Args) -> anyhow::Result<()> {
        let v = v.unwrap_value();
        args.dfa_size_limit = Some(convert::human_readable_usize(&v)?);
        Ok(())
    }
}

#[cfg(test)]
#[test]
fn test_dfa_size_limit() {
    let args = parse(None::<&str>).unwrap();
    assert_eq!(None, args.dfa_size_limit);

    #[cfg(target_pointer_width = "64")]
    {
        let args = parse(["--dfa-size-limit", "9G"]).unwrap();
        assert_eq!(Some(9 * (1 << 30)), args.dfa_size_limit);

        let args = parse(["--dfa-size-limit=9G"]).unwrap();
        assert_eq!(Some(9 * (1 << 30)), args.dfa_size_limit);

        let args =
            parse(["--dfa-size-limit=9G", "--dfa-size-limit=0"]).unwrap();
        assert_eq!(Some(0), args.dfa_size_limit);
    }

    let args = parse(["--dfa-size-limit=0K"]).unwrap();
    assert_eq!(Some(0), args.dfa_size_limit);

    let args = parse(["--dfa-size-limit=0M"]).unwrap();
    assert_eq!(Some(0), args.dfa_size_limit);

    let args = parse(["--dfa-size-limit=0G"]).unwrap();
    assert_eq!(Some(0), args.dfa_size_limit);

    let result = parse(["--dfa-size-limit", "9999999999999999999999"]);
    assert!(result.is_err(), "{result:?}");

    let result = parse(["--dfa-size-limit", "9999999999999999G"]);
    assert!(result.is_err(), "{result:?}");
}

/// -E/--encoding
#[derive(Debug)]
struct Encoding;

impl Flag for Encoding {
    fn is_switch(&self) -> bool {
        false
    }
    fn name_short(&self) -> Option<u8> {
        Some(b'E')
    }
    fn name_long(&self) -> &'static str {
        "encoding"
    }
    fn name_negated(&self) -> Option<&'static str> {
        Some("no-encoding")
    }
    fn doc_variable(&self) -> Option<&'static str> {
        Some("ENCODING")
    }
    fn doc_category(&self) -> &'static str {
        "search"
    }
    fn doc_short(&self) -> &'static str {
        r"Specify the text encoding of files to search."
    }
    fn doc_long(&self) -> &'static str {
        r"
Specify the text encoding that ripgrep will use on all files searched. The
default value is \fBauto\fP, which will cause ripgrep to do a best effort
automatic detection of encoding on a per-file basis. Automatic detection in
this case only applies to files that begin with a UTF-8 or UTF-16 byte-order
mark (BOM). No other automatic detection is performed. One can also specify
\fBnone\fP which will then completely disable BOM sniffing and always result
in searching the raw bytes, including a BOM if it's present, regardless of its
encoding.
.sp
Other supported values can be found in the list of labels here:
\fIhttps://encoding.spec.whatwg.org/#concept-encoding-get\fP.
.sp
For more details on encoding and how ripgrep deals with it, see \fBGUIDE.md\fP.
"
    }

    fn update(&self, v: FlagValue, args: &mut Args) -> anyhow::Result<()> {
        let value = match v {
            FlagValue::Value(v) => v,
            FlagValue::Switch(true) => {
                unreachable!("--encoding must accept a value")
            }
            FlagValue::Switch(false) => {
                args.encoding = EncodingMode::Auto;
                return Ok(());
            }
        };
        let label = convert::str(&value)?;
        args.encoding = match label {
            "auto" => EncodingMode::Auto,
            "none" => EncodingMode::Disabled,
            _ => EncodingMode::Some(grep::searcher::Encoding::new(label)?),
        };
        Ok(())
    }
}

#[cfg(test)]
#[test]
fn test_encoding() {
    let args = parse(None::<&str>).unwrap();
    assert_eq!(EncodingMode::Auto, args.encoding);

    let args = parse(["--encoding", "auto"]).unwrap();
    assert_eq!(EncodingMode::Auto, args.encoding);

    let args = parse(["--encoding", "none"]).unwrap();
    assert_eq!(EncodingMode::Disabled, args.encoding);

    let args = parse(["--encoding=none"]).unwrap();
    assert_eq!(EncodingMode::Disabled, args.encoding);

    let args = parse(["-E", "none"]).unwrap();
    assert_eq!(EncodingMode::Disabled, args.encoding);

    let args = parse(["-Enone"]).unwrap();
    assert_eq!(EncodingMode::Disabled, args.encoding);

    let args = parse(["-E", "none", "--no-encoding"]).unwrap();
    assert_eq!(EncodingMode::Auto, args.encoding);

    let args = parse(["--no-encoding", "-E", "none"]).unwrap();
    assert_eq!(EncodingMode::Disabled, args.encoding);

    let args = parse(["-E", "utf-16"]).unwrap();
    let enc = grep::searcher::Encoding::new("utf-16").unwrap();
    assert_eq!(EncodingMode::Some(enc), args.encoding);

    let args = parse(["-E", "utf-16", "--no-encoding"]).unwrap();
    assert_eq!(EncodingMode::Auto, args.encoding);

    let result = parse(["-E", "foo"]);
    assert!(result.is_err(), "{result:?}");
}

/// --engine
#[derive(Debug)]
struct Engine;

impl Flag for Engine {
    fn is_switch(&self) -> bool {
        false
    }
    fn name_long(&self) -> &'static str {
        "engine"
    }
    fn doc_variable(&self) -> Option<&'static str> {
        Some("ENGINE")
    }
    fn doc_category(&self) -> &'static str {
        "search"
    }
    fn doc_short(&self) -> &'static str {
        r"Specify which regex engine to use."
    }
    fn doc_long(&self) -> &'static str {
        r"
Specify which regular expression engine to use. When you choose a regex engine,
it applies that choice for every regex provided to ripgrep (e.g., via multiple
\flag{regexp} or \flag{file} flags).
.sp
Accepted values are \fBdefault\fP, \fBpcre2\fP, or \fBauto\fP.
.sp
The default value is \fBdefault\fP, which is usually the fastest and should be
good for most use cases. The \fBpcre2\fP engine is generally useful when you
want to use features such as look-around or backreferences. \fBauto\fP will
dynamically choose between supported regex engines depending on the features
used in a pattern on a best effort basis.
.sp
Note that the \fBpcre2\fP engine is an optional ripgrep feature. If PCRE2
wasn't included in your build of ripgrep, then using this flag will result in
ripgrep printing an error message and exiting.
.sp
This overrides previous uses of the \flag{pcre2} and \flag{auto-hybrid-regex}
flags.
"
    }
    fn doc_choices(&self) -> &'static [&'static str] {
        &["default", "pcre2", "auto"]
    }

    fn update(&self, v: FlagValue, args: &mut Args) -> anyhow::Result<()> {
        let v = v.unwrap_value();
        let string = convert::str(&v)?;
        args.engine = match string {
            "default" => EngineChoice::Default,
            "pcre2" => EngineChoice::PCRE2,
            "auto" => EngineChoice::Auto,
            _ => anyhow::bail!("unrecognized regex engine '{string}'"),
        };
        Ok(())
    }
}

#[cfg(test)]
#[test]
fn test_engine() {
    let args = parse(None::<&str>).unwrap();
    assert_eq!(EngineChoice::Default, args.engine);

    let args = parse(["--engine", "pcre2"]).unwrap();
    assert_eq!(EngineChoice::PCRE2, args.engine);

    let args = parse(["--engine=pcre2"]).unwrap();
    assert_eq!(EngineChoice::PCRE2, args.engine);

    let args = parse(["--auto-hybrid-regex", "--engine=pcre2"]).unwrap();
    assert_eq!(EngineChoice::PCRE2, args.engine);

    let args = parse(["--engine=pcre2", "--auto-hybrid-regex"]).unwrap();
    assert_eq!(EngineChoice::Auto, args.engine);

    let args = parse(["--auto-hybrid-regex", "--engine=auto"]).unwrap();
    assert_eq!(EngineChoice::Auto, args.engine);

    let args = parse(["--auto-hybrid-regex", "--engine=default"]).unwrap();
    assert_eq!(EngineChoice::Default, args.engine);

    let args = parse(["--engine=pcre2", "--no-auto-hybrid-regex"]).unwrap();
    assert_eq!(EngineChoice::Default, args.engine);
}

/// --field-context-separator
#[derive(Debug)]
struct FieldContextSeparator;

impl Flag for FieldContextSeparator {
    fn is_switch(&self) -> bool {
        false
    }
    fn name_long(&self) -> &'static str {
        "field-context-separator"
    }
    fn doc_variable(&self) -> Option<&'static str> {
        Some("SEPARATOR")
    }
    fn doc_category(&self) -> &'static str {
        "output"
    }
    fn doc_short(&self) -> &'static str {
        r"Set the field context separator."
    }
    fn doc_long(&self) -> &'static str {
        r"
Set the field context separator. This separator is only used when printing
contextual lines. It is used to delimit file paths, line numbers, columns and
the contextual line itself. The separator may be any number of bytes, including
zero. Escape sequences like \fB\\x7F\fP or \fB\\t\fP may be used.
.sp
The \fB-\fP character is the default value.
"
    }

    fn update(&self, v: FlagValue, args: &mut Args) -> anyhow::Result<()> {
        use crate::flags::FieldContextSeparator as Separator;

        args.field_context_separator = Separator::new(&v.unwrap_value())?;
        Ok(())
    }
}

#[cfg(test)]
#[test]
fn test_field_context_separator() {
    use bstr::BString;

    let args = parse(None::<&str>).unwrap();
    assert_eq!(BString::from("-"), args.field_context_separator.into_bytes());

    let args = parse(["--field-context-separator", "XYZ"]).unwrap();
    assert_eq!(
        BString::from("XYZ"),
        args.field_context_separator.into_bytes()
    );

    let args = parse(["--field-context-separator=XYZ"]).unwrap();
    assert_eq!(
        BString::from("XYZ"),
        args.field_context_separator.into_bytes()
    );

    let args = parse([
        "--field-context-separator",
        "XYZ",
        "--field-context-separator",
        "ABC",
    ])
    .unwrap();
    assert_eq!(
        BString::from("ABC"),
        args.field_context_separator.into_bytes()
    );

    let args = parse(["--field-context-separator", r"\t"]).unwrap();
    assert_eq!(BString::from("\t"), args.field_context_separator.into_bytes());

    let args = parse(["--field-context-separator", r"\x00"]).unwrap();
    assert_eq!(
        BString::from("\x00"),
        args.field_context_separator.into_bytes()
    );

    // This checks that invalid UTF-8 can be used. This case isn't too tricky
    // to handle, because it passes the invalid UTF-8 as an escape sequence
    // that is itself valid UTF-8. It doesn't become invalid UTF-8 until after
    // the argument is parsed and then unescaped.
    let args = parse(["--field-context-separator", r"\xFF"]).unwrap();
    assert_eq!(
        BString::from(b"\xFF"),
        args.field_context_separator.into_bytes()
    );

    // In this case, we specifically try to pass an invalid UTF-8 argument to
    // the flag. In theory we might be able to support this, but because we do
    // unescaping and because unescaping wants valid UTF-8, we do a UTF-8 check
    // on the value. Since we pass invalid UTF-8, it fails. This demonstrates
    // that the only way to use an invalid UTF-8 separator is by specifying an
    // escape sequence that is itself valid UTF-8.
    #[cfg(unix)]
    {
        use std::{ffi::OsStr, os::unix::ffi::OsStrExt};

        let result = parse([
            OsStr::from_bytes(b"--field-context-separator"),
            OsStr::from_bytes(&[0xFF]),
        ]);
        assert!(result.is_err(), "{result:?}");
    }
}

/// --field-match-separator
#[derive(Debug)]
struct FieldMatchSeparator;

impl Flag for FieldMatchSeparator {
    fn is_switch(&self) -> bool {
        false
    }
    fn name_long(&self) -> &'static str {
        "field-match-separator"
    }
    fn doc_variable(&self) -> Option<&'static str> {
        Some("SEPARATOR")
    }
    fn doc_category(&self) -> &'static str {
        "output"
    }
    fn doc_short(&self) -> &'static str {
        r"Set the field match separator."
    }
    fn doc_long(&self) -> &'static str {
        r"
Set the field match separator. This separator is only used when printing
matching lines. It is used to delimit file paths, line numbers, columns and the
matching line itself. The separator may be any number of bytes, including zero.
Escape sequences like \fB\\x7F\fP or \fB\\t\fP may be used.
.sp
The \fB:\fP character is the default value.
"
    }

    fn update(&self, v: FlagValue, args: &mut Args) -> anyhow::Result<()> {
        use crate::flags::FieldMatchSeparator as Separator;

        args.field_match_separator = Separator::new(&v.unwrap_value())?;
        Ok(())
    }
}

#[cfg(test)]
#[test]
fn test_field_match_separator() {
    use bstr::BString;

    let args = parse(None::<&str>).unwrap();
    assert_eq!(BString::from(":"), args.field_match_separator.into_bytes());

    let args = parse(["--field-match-separator", "XYZ"]).unwrap();
    assert_eq!(BString::from("XYZ"), args.field_match_separator.into_bytes());

    let args = parse(["--field-match-separator=XYZ"]).unwrap();
    assert_eq!(BString::from("XYZ"), args.field_match_separator.into_bytes());

    let args = parse([
        "--field-match-separator",
        "XYZ",
        "--field-match-separator",
        "ABC",
    ])
    .unwrap();
    assert_eq!(BString::from("ABC"), args.field_match_separator.into_bytes());

    let args = parse(["--field-match-separator", r"\t"]).unwrap();
    assert_eq!(BString::from("\t"), args.field_match_separator.into_bytes());

    let args = parse(["--field-match-separator", r"\x00"]).unwrap();
    assert_eq!(BString::from("\x00"), args.field_match_separator.into_bytes());

    // This checks that invalid UTF-8 can be used. This case isn't too tricky
    // to handle, because it passes the invalid UTF-8 as an escape sequence
    // that is itself valid UTF-8. It doesn't become invalid UTF-8 until after
    // the argument is parsed and then unescaped.
    let args = parse(["--field-match-separator", r"\xFF"]).unwrap();
    assert_eq!(
        BString::from(b"\xFF"),
        args.field_match_separator.into_bytes()
    );

    // In this case, we specifically try to pass an invalid UTF-8 argument to
    // the flag. In theory we might be able to support this, but because we do
    // unescaping and because unescaping wants valid UTF-8, we do a UTF-8 check
    // on the value. Since we pass invalid UTF-8, it fails. This demonstrates
    // that the only way to use an invalid UTF-8 separator is by specifying an
    // escape sequence that is itself valid UTF-8.
    #[cfg(unix)]
    {
        use std::{ffi::OsStr, os::unix::ffi::OsStrExt};

        let result = parse([
            OsStr::from_bytes(b"--field-match-separator"),
            OsStr::from_bytes(&[0xFF]),
        ]);
        assert!(result.is_err(), "{result:?}");
    }
}

/// -f/--file
#[derive(Debug)]
struct File;

impl Flag for File {
    fn is_switch(&self) -> bool {
        false
    }
    fn name_short(&self) -> Option<u8> {
        Some(b'f')
    }
    fn name_long(&self) -> &'static str {
        "file"
    }
    fn doc_variable(&self) -> Option<&'static str> {
        Some("PATTERNFILE")
    }
    fn doc_category(&self) -> &'static str {
        "input"
    }
    fn doc_short(&self) -> &'static str {
        r"Search for patterns from the given file."
    }
    fn doc_long(&self) -> &'static str {
        r"
Search for patterns from the given file, with one pattern per line. When this
flag is used multiple times or in combination with the \flag{regexp} flag, then
all patterns provided are searched. Empty pattern lines will match all input
lines, and the newline is not counted as part of the pattern.
.sp
A line is printed if and only if it matches at least one of the patterns.
.sp
When \fIPATTERNFILE\fP is \fB-\fP, then \fBstdin\fP will be read for the
patterns.
.sp
When \flag{file} or \flag{regexp} is used, then ripgrep treats all positional
arguments as files or directories to search.
"
    }

    fn update(&self, v: FlagValue, args: &mut Args) -> anyhow::Result<()> {
        let path = PathBuf::from(v.unwrap_value());
        args.patterns.push(PatternSource::File(path));
        Ok(())
    }
}

#[cfg(test)]
#[test]
fn test_file() {
    let args = parse(None::<&str>).unwrap();
    assert_eq!(Vec::<PatternSource>::new(), args.patterns);

    let args = parse(["--file", "foo"]).unwrap();
    assert_eq!(vec![PatternSource::File(PathBuf::from("foo"))], args.patterns);

    let args = parse(["--file=foo"]).unwrap();
    assert_eq!(vec![PatternSource::File(PathBuf::from("foo"))], args.patterns);

    let args = parse(["-f", "foo"]).unwrap();
    assert_eq!(vec![PatternSource::File(PathBuf::from("foo"))], args.patterns);

    let args = parse(["-ffoo"]).unwrap();
    assert_eq!(vec![PatternSource::File(PathBuf::from("foo"))], args.patterns);

    let args = parse(["--file", "-foo"]).unwrap();
    assert_eq!(
        vec![PatternSource::File(PathBuf::from("-foo"))],
        args.patterns
    );

    let args = parse(["--file=-foo"]).unwrap();
    assert_eq!(
        vec![PatternSource::File(PathBuf::from("-foo"))],
        args.patterns
    );

    let args = parse(["-f", "-foo"]).unwrap();
    assert_eq!(
        vec![PatternSource::File(PathBuf::from("-foo"))],
        args.patterns
    );

    let args = parse(["-f-foo"]).unwrap();
    assert_eq!(
        vec![PatternSource::File(PathBuf::from("-foo"))],
        args.patterns
    );

    let args = parse(["--file=foo", "--file", "bar"]).unwrap();
    assert_eq!(
        vec![
            PatternSource::File(PathBuf::from("foo")),
            PatternSource::File(PathBuf::from("bar"))
        ],
        args.patterns
    );

    // We permit path arguments to be invalid UTF-8. So test that. Some of
    // these cases are tricky and depend on lexopt doing the right thing.
    //
    // We probably should add tests for this handling on Windows too, but paths
    // that are invalid UTF-16 appear incredibly rare in the Windows world.
    #[cfg(unix)]
    {
        use std::{
            ffi::{OsStr, OsString},
            os::unix::ffi::{OsStrExt, OsStringExt},
        };

        let bytes = &[b'A', 0xFF, b'Z'][..];
        let path = PathBuf::from(OsString::from_vec(bytes.to_vec()));

        let args =
            parse([OsStr::from_bytes(b"--file"), OsStr::from_bytes(bytes)])
                .unwrap();
        assert_eq!(vec![PatternSource::File(path.clone())], args.patterns);

        let args = parse([OsStr::from_bytes(b"-f"), OsStr::from_bytes(bytes)])
            .unwrap();
        assert_eq!(vec![PatternSource::File(path.clone())], args.patterns);

        let mut bytes = b"--file=A".to_vec();
        bytes.push(0xFF);
        bytes.push(b'Z');
        let args = parse([OsStr::from_bytes(&bytes)]).unwrap();
        assert_eq!(vec![PatternSource::File(path.clone())], args.patterns);

        let mut bytes = b"-fA".to_vec();
        bytes.push(0xFF);
        bytes.push(b'Z');
        let args = parse([OsStr::from_bytes(&bytes)]).unwrap();
        assert_eq!(vec![PatternSource::File(path.clone())], args.patterns);
    }
}

/// --files
#[derive(Debug)]
struct Files;

impl Flag for Files {
    fn is_switch(&self) -> bool {
        true
    }
    fn name_long(&self) -> &'static str {
        "files"
    }
    fn doc_category(&self) -> &'static str {
        "other-behaviors"
    }
    fn doc_short(&self) -> &'static str {
        r"Print each file that would be searched."
    }
    fn doc_long(&self) -> &'static str {
        r"
Print each file that would be searched without actually performing the search.
This is useful to determine whether a particular file is being searched or not.
.sp
This overrides \flag{type-list}.
"
    }

    fn update(&self, v: FlagValue, args: &mut Args) -> anyhow::Result<()> {
        assert!(v.unwrap_switch());
        args.files = true;
        args.type_list = false;
        Ok(())
    }
}

#[cfg(test)]
#[test]
fn test_files() {
    let args = parse(None::<&str>).unwrap();
    assert_eq!(false, args.files);

    let args = parse(["--files"]).unwrap();
    assert_eq!(true, args.files);
}

/// -l/--files-with-matches
#[derive(Debug)]
struct FilesWithMatches;

impl Flag for FilesWithMatches {
    fn is_switch(&self) -> bool {
        true
    }
    fn name_short(&self) -> Option<u8> {
        Some(b'l')
    }
    fn name_long(&self) -> &'static str {
        "files-with-matches"
    }
    fn doc_category(&self) -> &'static str {
        "output-modes"
    }
    fn doc_short(&self) -> &'static str {
        r"Print the paths with at least one match."
    }
    fn doc_long(&self) -> &'static str {
        r"
Print only the paths with at least one match and suppress match contents.
.sp
This overrides \flag{files-without-match}.
"
    }

    fn update(&self, v: FlagValue, args: &mut Args) -> anyhow::Result<()> {
        assert!(v.unwrap_switch(), "--files-with-matches can only be enabled");
        args.file_matches = Some(FileMatchMode::WithMatches);
        Ok(())
    }
}

#[cfg(test)]
#[test]
fn test_files_with_matches() {
    let args = parse(None::<&str>).unwrap();
    assert_eq!(None, args.file_matches);

    let args = parse(["--files-with-matches"]).unwrap();
    assert_eq!(Some(FileMatchMode::WithMatches), args.file_matches);

    let args = parse(["-l"]).unwrap();
    assert_eq!(Some(FileMatchMode::WithMatches), args.file_matches);
}

/// -l/--files-without-match
#[derive(Debug)]
struct FilesWithoutMatch;

impl Flag for FilesWithoutMatch {
    fn is_switch(&self) -> bool {
        true
    }
    fn name_long(&self) -> &'static str {
        "files-without-match"
    }
    fn doc_category(&self) -> &'static str {
        "output-modes"
    }
    fn doc_short(&self) -> &'static str {
        r"Print the paths that contain zero matches."
    }
    fn doc_long(&self) -> &'static str {
        r"
Print the paths that contain zero matches and suppress match contents.
.sp
This overrides \flag{files-with-matches}.
"
    }

    fn update(&self, v: FlagValue, args: &mut Args) -> anyhow::Result<()> {
        assert!(
            v.unwrap_switch(),
            "--files-without-match can only be enabled"
        );
        args.file_matches = Some(FileMatchMode::WithoutMatch);
        Ok(())
    }
}

#[cfg(test)]
#[test]
fn test_files_without_match() {
    let args = parse(None::<&str>).unwrap();
    assert_eq!(None, args.file_matches);

    let args = parse(["--files-without-match"]).unwrap();
    assert_eq!(Some(FileMatchMode::WithoutMatch), args.file_matches);

    let args =
        parse(["--files-with-matches", "--files-without-match"]).unwrap();
    assert_eq!(Some(FileMatchMode::WithoutMatch), args.file_matches);

    let args =
        parse(["--files-without-match", "--files-with-matches"]).unwrap();
    assert_eq!(Some(FileMatchMode::WithMatches), args.file_matches);
}

/// -F/--fixed-strings
#[derive(Debug)]
struct FixedStrings;

impl Flag for FixedStrings {
    fn is_switch(&self) -> bool {
        true
    }
    fn name_short(&self) -> Option<u8> {
        Some(b'F')
    }
    fn name_long(&self) -> &'static str {
        "fixed-strings"
    }
    fn name_negated(&self) -> Option<&'static str> {
        Some("no-fixed-strings")
    }
    fn doc_category(&self) -> &'static str {
        "search"
    }
    fn doc_short(&self) -> &'static str {
        r"Treat all patterns as literals."
    }
    fn doc_long(&self) -> &'static str {
        r"
Treat all patterns as literals instead of as regular expressions. When this
flag is used, special regular expression meta characters such as \fB.(){}*+\fP
should not need be escaped.
"
    }

    fn update(&self, v: FlagValue, args: &mut Args) -> anyhow::Result<()> {
        args.fixed_strings = v.unwrap_switch();
        Ok(())
    }
}

#[cfg(test)]
#[test]
fn test_fixed_strings() {
    let args = parse(None::<&str>).unwrap();
    assert_eq!(false, args.fixed_strings);

    let args = parse(["--fixed-strings"]).unwrap();
    assert_eq!(true, args.fixed_strings);

    let args = parse(["-F"]).unwrap();
    assert_eq!(true, args.fixed_strings);

    let args = parse(["-F", "--no-fixed-strings"]).unwrap();
    assert_eq!(false, args.fixed_strings);

    let args = parse(["--no-fixed-strings", "-F"]).unwrap();
    assert_eq!(true, args.fixed_strings);
}

/// -L/--follow
#[derive(Debug)]
struct Follow;

impl Flag for Follow {
    fn is_switch(&self) -> bool {
        true
    }
    fn name_short(&self) -> Option<u8> {
        Some(b'L')
    }
    fn name_long(&self) -> &'static str {
        "follow"
    }
    fn name_negated(&self) -> Option<&'static str> {
        Some("no-follow")
    }
    fn doc_category(&self) -> &'static str {
        "filter"
    }
    fn doc_short(&self) -> &'static str {
        r"Follow symbolic links."
    }
    fn doc_long(&self) -> &'static str {
        r"
This flag instructs ripgrep to follow symbolic links while traversing
directories. This behavior is disabled by default. Note that ripgrep will
check for symbolic link loops and report errors if it finds one. ripgrep will
also report errors for broken links. To suppress error messages, use the
\flag{no-messages} flag.
"
    }

    fn update(&self, v: FlagValue, args: &mut Args) -> anyhow::Result<()> {
        args.follow = v.unwrap_switch();
        Ok(())
    }
}

#[cfg(test)]
#[test]
fn test_follow() {
    let args = parse(None::<&str>).unwrap();
    assert_eq!(false, args.follow);

    let args = parse(["--follow"]).unwrap();
    assert_eq!(true, args.follow);

    let args = parse(["-L"]).unwrap();
    assert_eq!(true, args.follow);

    let args = parse(["-L", "--no-follow"]).unwrap();
    assert_eq!(false, args.follow);

    let args = parse(["--no-follow", "-L"]).unwrap();
    assert_eq!(true, args.follow);
}

/// -g/--glob
#[derive(Debug)]
struct Glob;

impl Flag for Glob {
    fn is_switch(&self) -> bool {
        false
    }
    fn name_short(&self) -> Option<u8> {
        Some(b'g')
    }
    fn name_long(&self) -> &'static str {
        "glob"
    }
    fn doc_variable(&self) -> Option<&'static str> {
        Some("GLOB")
    }
    fn doc_category(&self) -> &'static str {
        "filter"
    }
    fn doc_short(&self) -> &'static str {
        r"Include or exclude file paths."
    }
    fn doc_long(&self) -> &'static str {
        r#"
Include or exclude files and directories for searching that match the given
glob. This always overrides any other ignore logic. Multiple glob flags may
be used. Globbing rules match \fB.gitignore\fP globs. Precede a glob with a
\fB!\fP to exclude it. If multiple globs match a file or directory, the glob
given later in the command line takes precedence.
.sp
As an extension, globs support specifying alternatives:
.BI "\-g '" ab{c,d}* '
is
equivalent to
.BI "\-g " "abc " "\-g " abd.
Empty alternatives like
.BI "\-g '" ab{,c} '
are not currently supported. Note that this syntax extension is also currently
enabled in \fBgitignore\fP files, even though this syntax isn't supported by
git itself. ripgrep may disable this syntax extension in gitignore files, but
it will always remain available via the \flag{glob} flag.
.sp
When this flag is set, every file and directory is applied to it to test for
a match. For example, if you only want to search in a particular directory
\fIfoo\fP, then
.BI "\-g " foo
is incorrect because \fIfoo/bar\fP does not match
the glob \fIfoo\fP. Instead, you should use
.BI "\-g '" foo/** '.
"#
    }

    fn update(&self, v: FlagValue, args: &mut Args) -> anyhow::Result<()> {
        let glob = convert::string(v.unwrap_value())?;
        args.globs.push(glob);
        Ok(())
    }
}

#[cfg(test)]
#[test]
fn test_glob() {
    let args = parse(None::<&str>).unwrap();
    assert_eq!(Vec::<String>::new(), args.globs);

    let args = parse(["--glob", "foo"]).unwrap();
    assert_eq!(vec!["foo".to_string()], args.globs);

    let args = parse(["--glob=foo"]).unwrap();
    assert_eq!(vec!["foo".to_string()], args.globs);

    let args = parse(["-g", "foo"]).unwrap();
    assert_eq!(vec!["foo".to_string()], args.globs);

    let args = parse(["-gfoo"]).unwrap();
    assert_eq!(vec!["foo".to_string()], args.globs);

    let args = parse(["--glob", "-foo"]).unwrap();
    assert_eq!(vec!["-foo".to_string()], args.globs);

    let args = parse(["--glob=-foo"]).unwrap();
    assert_eq!(vec!["-foo".to_string()], args.globs);

    let args = parse(["-g", "-foo"]).unwrap();
    assert_eq!(vec!["-foo".to_string()], args.globs);

    let args = parse(["-g-foo"]).unwrap();
    assert_eq!(vec!["-foo".to_string()], args.globs);
}

/// --glob-case-insensitive
#[derive(Debug)]
struct GlobCaseInsensitive;

impl Flag for GlobCaseInsensitive {
    fn is_switch(&self) -> bool {
        true
    }
    fn name_long(&self) -> &'static str {
        "glob-case-insensitive"
    }
    fn name_negated(&self) -> Option<&'static str> {
        Some("no-glob-case-insensitive")
    }
    fn doc_category(&self) -> &'static str {
        "filter"
    }
    fn doc_short(&self) -> &'static str {
        r"Process all glob patterns case insensitively."
    }
    fn doc_long(&self) -> &'static str {
        r"
Process all glob patterns given with the \flag{glob} flag case insensitively.
This effectively treats \flag{glob} as \flag{iglob}.
"
    }

    fn update(&self, v: FlagValue, args: &mut Args) -> anyhow::Result<()> {
        args.glob_case_insensitive = v.unwrap_switch();
        Ok(())
    }
}

#[cfg(test)]
#[test]
fn test_glob_case_insensitive() {
    let args = parse(None::<&str>).unwrap();
    assert_eq!(false, args.glob_case_insensitive);

    let args = parse(["--glob-case-insensitive"]).unwrap();
    assert_eq!(true, args.glob_case_insensitive);

    let args =
        parse(["--glob-case-insensitive", "--no-glob-case-insensitive"])
            .unwrap();
    assert_eq!(false, args.glob_case_insensitive);

    let args =
        parse(["--no-glob-case-insensitive", "--glob-case-insensitive"])
            .unwrap();
    assert_eq!(true, args.glob_case_insensitive);
}

/// --heading
#[derive(Debug)]
struct Heading;

impl Flag for Heading {
    fn is_switch(&self) -> bool {
        true
    }
    fn name_long(&self) -> &'static str {
        "heading"
    }
    fn name_negated(&self) -> Option<&'static str> {
        Some("no-heading")
    }
    fn doc_category(&self) -> &'static str {
        "output"
    }
    fn doc_short(&self) -> &'static str {
        r"Print matches grouped by each file."
    }
    fn doc_long(&self) -> &'static str {
        r"
This flag prints the file path above clusters of matches from each file instead
of printing the file path as a prefix for each matched line. This is the
default mode when printing to a terminal.
.sp
When \fBstdout\fP is not a terminal, then ripgrep will default to the standard
grep-like format. Once can force this format in Unix-like environments by
piping the output of ripgrep to \fBcat\fP. For example, \fBrg\fP \fIfoo\fP \fB|
cat\fP.
"
    }

    fn update(&self, v: FlagValue, args: &mut Args) -> anyhow::Result<()> {
        args.heading = Some(v.unwrap_switch());
        Ok(())
    }
}

#[cfg(test)]
#[test]
fn test_heading() {
    let args = parse(None::<&str>).unwrap();
    assert_eq!(None, args.heading);

    let args = parse(["--heading"]).unwrap();
    assert_eq!(Some(true), args.heading);

    let args = parse(["--no-heading"]).unwrap();
    assert_eq!(Some(false), args.heading);

    let args = parse(["--heading", "--no-heading"]).unwrap();
    assert_eq!(Some(false), args.heading);

    let args = parse(["--no-heading", "--heading"]).unwrap();
    assert_eq!(Some(true), args.heading);
}

/// -e/--regexp
#[derive(Debug)]
struct Regexp;

impl Flag for Regexp {
    fn is_switch(&self) -> bool {
        false
    }
    fn name_short(&self) -> Option<u8> {
        Some(b'e')
    }
    fn name_long(&self) -> &'static str {
        "regexp"
    }
    fn doc_variable(&self) -> Option<&'static str> {
        Some("PATTERN")
    }
    fn doc_category(&self) -> &'static str {
        "input"
    }
    fn doc_short(&self) -> &'static str {
        r"A pattern to search for."
    }
    fn doc_long(&self) -> &'static str {
        r"
A pattern to search for. This option can be provided multiple times, where
all patterns given are searched, in addition to any patterns provided by
\flag{file}. Lines matching at least one of the provided patterns are printed.
This flag can also be used when searching for patterns that start with a dash.
.sp
For example, to search for the literal \fB\-foo\fP:
.sp
.EX
    rg \-e \-foo
.EE
.sp
You can also use the special \fB\-\-\fP delimiter to indicate that no more
flags will be provided. Namely, the following is equivalent to the above:
.sp
.EX
    rg \-\- \-foo
.EE
.sp
When \flag{file} or \flag{regexp} is used, then ripgrep treats all positional
arguments as files or directories to search.
"
    }

    fn update(&self, v: FlagValue, args: &mut Args) -> anyhow::Result<()> {
        let regexp = convert::string(v.unwrap_value())?;
        args.patterns.push(PatternSource::Regexp(regexp));
        Ok(())
    }
}

#[cfg(test)]
#[test]
fn test_regexp() {
    let args = parse(None::<&str>).unwrap();
    assert_eq!(Vec::<PatternSource>::new(), args.patterns);

    let args = parse(["--regexp", "foo"]).unwrap();
    assert_eq!(vec![PatternSource::Regexp("foo".to_string())], args.patterns);

    let args = parse(["--regexp=foo"]).unwrap();
    assert_eq!(vec![PatternSource::Regexp("foo".to_string())], args.patterns);

    let args = parse(["-e", "foo"]).unwrap();
    assert_eq!(vec![PatternSource::Regexp("foo".to_string())], args.patterns);

    let args = parse(["-efoo"]).unwrap();
    assert_eq!(vec![PatternSource::Regexp("foo".to_string())], args.patterns);

    let args = parse(["--regexp", "-foo"]).unwrap();
    assert_eq!(vec![PatternSource::Regexp("-foo".to_string())], args.patterns);

    let args = parse(["--regexp=-foo"]).unwrap();
    assert_eq!(vec![PatternSource::Regexp("-foo".to_string())], args.patterns);

    let args = parse(["-e", "-foo"]).unwrap();
    assert_eq!(vec![PatternSource::Regexp("-foo".to_string())], args.patterns);

    let args = parse(["-e-foo"]).unwrap();
    assert_eq!(vec![PatternSource::Regexp("-foo".to_string())], args.patterns);

    let args = parse(["--regexp=foo", "--regexp", "bar"]).unwrap();
    assert_eq!(
        vec![
            PatternSource::Regexp("foo".to_string()),
            PatternSource::Regexp("bar".to_string())
        ],
        args.patterns
    );

    // While we support invalid UTF-8 arguments in general, patterns must be
    // valid UTF-8.
    #[cfg(unix)]
    {
        use std::{
            ffi::{OsStr, OsString},
            os::unix::ffi::{OsStrExt, OsStringExt},
        };

        let bytes = &[b'A', 0xFF, b'Z'][..];
        let result =
            parse([OsStr::from_bytes(b"-e"), OsStr::from_bytes(bytes)]);
        assert!(result.is_err(), "{result:?}");
    }

    // Check that combining -e/--regexp and -f/--file works as expected.
    let args = parse(["-efoo", "-fbar"]).unwrap();
    assert_eq!(
        vec![
            PatternSource::Regexp("foo".to_string()),
            PatternSource::File(PathBuf::from("bar"))
        ],
        args.patterns
    );

    let args = parse(["-efoo", "-fbar", "-equux"]).unwrap();
    assert_eq!(
        vec![
            PatternSource::Regexp("foo".to_string()),
            PatternSource::File(PathBuf::from("bar")),
            PatternSource::Regexp("quux".to_string()),
        ],
        args.patterns
    );
}

/// --trace
#[derive(Debug)]
struct Trace;

impl Flag for Trace {
    fn is_switch(&self) -> bool {
        true
    }
    fn name_long(&self) -> &'static str {
        "trace"
    }
    fn doc_category(&self) -> &'static str {
        "logging"
    }
    fn doc_short(&self) -> &'static str {
        r"Show trace messages."
    }
    fn doc_long(&self) -> &'static str {
        r"
Show trace messages. This shows even more detail than the \flag{debug}
flag. Generally, one should only use this if \flag{debug} doesn't emit the
information you're looking for.
"
    }

    fn update(&self, v: FlagValue, args: &mut Args) -> anyhow::Result<()> {
        assert!(v.unwrap_switch(), "--trace can only be enabled");
        args.logging = Some(LoggingMode::Trace);
        Ok(())
    }
}

#[cfg(test)]
#[test]
fn test_trace() {
    let args = parse(None::<&str>).unwrap();
    assert_eq!(None, args.logging);

    let args = parse(["--trace"]).unwrap();
    assert_eq!(Some(LoggingMode::Trace), args.logging);

    let args = parse(["--debug", "--trace"]).unwrap();
    assert_eq!(Some(LoggingMode::Trace), args.logging);
}

mod convert {
    use std::ffi::{OsStr, OsString};

    use {anyhow::Context, bstr::ByteSlice};

    pub(super) fn str(v: &OsStr) -> anyhow::Result<&str> {
        let Some(s) = v.to_str() else {
            anyhow::bail!("value is not valid UTF-8")
        };
        Ok(s)
    }

    pub(super) fn string(v: OsString) -> anyhow::Result<String> {
        let Ok(s) = v.into_string() else {
            anyhow::bail!("value is not valid UTF-8")
        };
        Ok(s)
    }

    pub(super) fn usize(v: &OsStr) -> anyhow::Result<usize> {
        str(v)?.parse().context("value is not a valid number")
    }

    pub(super) fn human_readable_size(v: &OsStr) -> anyhow::Result<u64> {
        grep::cli::parse_human_readable_size(str(v)?).context("invalid size")
    }

    pub(super) fn human_readable_usize(v: &OsStr) -> anyhow::Result<usize> {
        let size = human_readable_size(v)?;
        let Ok(size) = usize::try_from(size) else {
            anyhow::bail!("size is too big")
        };
        Ok(size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn available_shorts() {
        let mut total = vec![false; 128];
        for byte in 0..=0x7F {
            match byte {
                b'.' | b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z' => {
                    total[usize::from(byte)] = true
                }
                _ => continue,
            }
        }

        let mut taken = vec![false; 128];
        for flag in FLAGS.iter() {
            let Some(short) = flag.name_short() else { continue };
            taken[usize::from(short)] = true;
        }

        for byte in 0..=0x7F {
            if total[usize::from(byte)] && !taken[usize::from(byte)] {
                eprintln!("{:?}", char::from(byte));
            }
        }
    }

    #[test]
    fn shorts_all_ascii_alphanumeric() {
        for flag in FLAGS.iter() {
            let Some(byte) = flag.name_short() else { continue };
            let long = flag.name_long();
            assert!(
                byte.is_ascii_alphanumeric() || byte == b'.',
                "\\x{byte:0X} is not a valid short flag for {long}",
            )
        }
    }

    #[test]
    fn longs_all_ascii_alphanumeric() {
        for flag in FLAGS.iter() {
            let long = flag.name_long();
            let count = long.chars().count();
            assert!(count >= 2, "flag '{long}' is less than 2 characters");
            assert!(
                long.chars().all(|c| c.is_ascii_alphanumeric() || c == '-'),
                "flag '{long}' does not match ^[-0-9A-Za-z]+$",
            );
            let Some(negated) = flag.name_negated() else { continue };
            assert!(
                negated.chars().all(|c| c.is_ascii_alphanumeric() || c == '-'),
                "flag '{negated}' does not match ^[-0-9A-Za-z]+$",
            );
        }
    }

    #[test]
    fn shorts_no_duplicates() {
        let mut taken = vec![false; 128];
        for flag in FLAGS.iter() {
            let Some(short) = flag.name_short() else { continue };
            let long = flag.name_long();
            assert!(
                !taken[usize::from(short)],
                "flag {long} has duplicate short flag {}",
                char::from(short)
            );
        }
    }

    #[test]
    fn longs_no_duplicates() {
        use std::collections::BTreeSet;

        let mut taken = BTreeSet::new();
        for flag in FLAGS.iter() {
            let long = flag.name_long();
            assert!(taken.insert(long), "flag {long} has a duplicate name");
            let Some(negated) = flag.name_negated() else { continue };
            assert!(
                taken.insert(negated),
                "negated flag {negated} has a duplicate name"
            );
        }
    }

    #[test]
    fn switches_have_no_choices() {
        for flag in FLAGS.iter() {
            if !flag.is_switch() {
                continue;
            }
            let long = flag.name_long();
            let choices = flag.doc_choices();
            assert!(
                choices.is_empty(),
                "switch flag '{long}' \
                 should not have any choices but has some: {choices:?}",
            );
        }
    }

    #[test]
    fn choices_ascii_alphanumeric() {
        for flag in FLAGS.iter() {
            let long = flag.name_long();
            for choice in flag.doc_choices() {
                assert!(
                    choice
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || c == '-'),
                    "choice '{choice}' for flag '{long}' does not match \
                     ^[-0-9A-Za-z]+$",
                )
            }
        }
    }
}
