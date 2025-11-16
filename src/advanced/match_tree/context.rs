#[derive(Clone, Copy)]
pub(crate) enum Context {
    Root,

    // MatchTree methods
    Parse(&'static str),
    AsSeq,
    AsAlt(usize),
    AsAltEnum(&'static str),
    AsOpt,

    // SeqMatch methods
    At(usize),
    Get(usize),
    ParseAt(&'static str, usize),
    ParseField(&'static str, usize, &'static str),
}

#[derive(Clone, Copy)]
pub(crate) struct ContextChain<'t> {
    current: Context,
    parent: Option<&'t ContextChain<'t>>,
}

impl<'t> ContextChain<'t> {
    pub fn and(&'t self, child: Context) -> Self {
        Self {
            current: child,
            parent: Some(self),
        }
    }
}

impl From<Context> for ContextChain<'_> {
    fn from(context: Context) -> Self {
        Self {
            current: context,
            parent: None,
        }
    }
}

impl std::fmt::Display for ContextChain<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(parent) = &self.parent {
            parent.fmt(f)?;
            f.write_str(" -> ")?;
        }
        match &self.current {
            Context::Root => f.write_str("sscanf"),

            Context::Parse(ty) => write!(f, "parse as {ty}"),
            Context::AsSeq => f.write_str("as_seq()"),
            Context::AsAlt(index) => write!(f, "as_alt({index} matched)"),
            Context::AsAltEnum(variant) => write!(f, "as_alt({variant} matched)"),
            Context::AsOpt => f.write_str("as_opt()"),

            Context::At(index) => write!(f, "at({index})"),
            Context::Get(index) => write!(f, "get({index})"),
            Context::ParseAt(ty, index) => write!(f, "parse {index} as {ty}"),
            Context::ParseField(name, index, ty) => {
                write!(f, "parse .{name} (index {index} as {ty})")
            }
        }
    }
}
