use std::ffi::OsString;

pub(crate) use crate::flags::{
    args::{
        Args, BinaryMode, BufferMode, CaseMode, ColorChoice, ContextMode,
        ContextSeparator, CountMode, EncodingMode, EngineChoice,
        FieldContextSeparator, FieldMatchSeparator, FileMatchMode,
        LoggingMode, MmapMode, PatternSource, SortMode, SortModeKind,
        TypeChoice,
    },
    complete::{
        bash::generate as generate_complete_bash,
        fish::generate as generate_complete_fish,
        powershell::generate as generate_complete_powershell,
    },
    doc::{
        help::{
            generate_long as generate_help_long,
            generate_short as generate_help_short,
        },
        man::generate as generate_man_page,
    },
    parse::parse,
};

mod args;
mod complete;
mod defs;
mod doc;
mod parse;

trait Flag: std::fmt::Debug + 'static {
    fn is_switch(&self) -> bool;
    fn name_short(&self) -> Option<u8> {
        None
    }
    fn name_long(&self) -> &'static str;
    fn name_negated(&self) -> Option<&'static str> {
        None
    }
    fn doc_variable(&self) -> Option<&'static str> {
        None
    }
    fn doc_category(&self) -> &'static str;
    fn doc_short(&self) -> &'static str;
    fn doc_long(&self) -> &'static str;
    fn doc_choices(&self) -> &'static [&'static str] {
        &[]
    }

    fn update(&self, value: FlagValue, args: &mut Args) -> anyhow::Result<()>;
}

enum FlagValue {
    Switch(bool),
    Value(OsString),
}

impl FlagValue {
    fn unwrap_switch(self) -> bool {
        match self {
            FlagValue::Switch(yes) => yes,
            FlagValue::Value(_) => {
                unreachable!("got flag value but expected switch")
            }
        }
    }

    fn unwrap_value(self) -> OsString {
        match self {
            FlagValue::Switch(yes) => {
                unreachable!("got switch but expected flag value")
            }
            FlagValue::Value(v) => v,
        }
    }
}
