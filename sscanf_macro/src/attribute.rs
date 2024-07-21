use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::hash::Hash;

use crate::*;

mod r#enum; // not all of these need to be r#, but this looks nicer
mod r#field;
mod r#struct;
mod r#variant;
pub use r#enum::*;
pub use r#field::*;
pub use r#struct::*;
pub use r#variant::*;

pub trait Attr: Debug + Display + Copy + Ord + Hash + 'static {
    fn all() -> &'static [Self];
    fn context() -> Context;

    fn as_str(&self) -> &'static str;
}

macro_rules! declare_attr {
    (
        $attr_mod: ident :: $attr_enum: ident {
            $($attr_ident: ident $attr_text: literal,)+
        },
        $context_enum: ident {
            $($context: ident $context_name: literal [ $($context_attr: ident),+ ],)+
        }
    ) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum $context_enum {
            $($context),+
        }
        #[allow(dead_code)]
        impl $context_enum {
            pub const ALL: &'static [Self] = &[ $(Self::$context),+ ];
            pub const ALL_NAMES: &'static [&'static str] = &[ $($context_name),+ ];
            pub const fn as_str(&self) -> &'static str {
                Self::ALL_NAMES[*self as usize]
            }
            pub const fn all_attr_names(&self) -> &'static [&'static str] {
                match self {
                    $(Self::$context => attr::$context::ALL_NAMES),+
                }
            }
        }
        impl std::fmt::Display for $context_enum {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.as_str())
            }
        }

        pub mod $attr_mod {
            pub enum $attr_enum {
                $($attr_ident),+
            }
            impl $attr_enum {
                pub const fn as_str(&self) -> &'static str {
                    match self {
                        $(Self::$attr_ident => $attr_text),+
                    }
                }
            }

            $(
                #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
                pub enum $context {
                    $($context_attr),+
                }
                #[allow(dead_code)]
                impl $context {
                    pub const ALL: &'static [Self] = &[$(Self::$context_attr),+];
                    pub const ALL_NAMES: &'static [&'static str] = &[$(Self::$context_attr.as_str()),+];
                    pub const fn as_str(&self) -> &'static str {
                        match self {
                            $(Self::$context_attr => $attr_enum::$context_attr.as_str()),+
                        }
                    }
                }
                impl std::fmt::Display for $context {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(f, "{}", self.as_str())
                    }
                }
                impl super::Attr for $context {
                    fn all() -> &'static [Self] {
                        Self::ALL
                    }
                    fn context() -> super::$context_enum {
                        super::$context_enum::$context
                    }
                    fn as_str(&self) -> &'static str {
                        self.as_str()
                    }
                }
            )+
        }

        use syn::punctuated::Punctuated;
        fn attr_parser<A: Attr>() -> fn(ParseStream) -> syn::Result<Punctuated<Attribute<A>, Token![,]>> {
            match A::context() {
                $($context_enum::$context => |input| Punctuated::parse_terminated_with(input, |input| Attribute::parse(input))),+
            }
        }
    };
}

declare_attr!(
    attr::All {
        // structs and variants
        Format "format",
        FormatUnescaped "format_unescaped",
        Transparent "transparent",
        // just variants
        Skip "skip",
        // enums
        AutoGen "autogen",
        AutoGenerate "autogenerate",
        // fields
        Default "default",
        Map "map",
        FilterMap "filter_map",
        From "from",
        TryFrom "try_from",
    },
    Context {
        Struct "structs" [ Format, FormatUnescaped, Transparent ],
        Variant "variants" [ Format, FormatUnescaped, Transparent, Skip ],
        Enum "enums" [ AutoGen, AutoGenerate ],
        Field "fields" [ Default, Map, FilterMap, From, TryFrom ],
    }
);

fn find_match<A: Attr>(s: &str, src: &TokenStream) -> syn::Result<A> {
    if let Some(attr) = A::all().iter().find(|attr| attr.as_str() == s) {
        return Ok(*attr);
    }

    let context = A::context();
    let valid = list_items(context.all_attr_names(), |s| format!("`{}`", s));

    let mut others = Context::ALL.to_vec();
    others.retain(|&other| other != context);

    let mut found_others = vec![];
    for other in &others {
        if other.all_attr_names().iter().any(|name| *name == s) {
            found_others.push(other);
        }
    }
    if !found_others.is_empty() {
        let others = list_items(&found_others, |other| other.to_string());
        let msg = format!(
            "attribute `{}` can only be used on {}.\n{} can have the following attributes: {}",
            s, others, context, valid
        );
        return Err(syn::Error::new_spanned(src, msg)); // checked in tests/fail/derive_struct_attributes.rs
    }

    if let Some(similar) = find_closest(s, context.all_attr_names()) {
        let msg = format!("unknown attribute `{}`. Did you mean `{}`?", s, similar);
        return Err(syn::Error::new_spanned(src, msg)); // checked in tests/fail/derive_struct_attributes.rs
    }

    for other in &others {
        if let Some(similar) = find_closest(s, other.all_attr_names()) {
            let msg = format!(
                "unknown attribute `{}` is similar to `{}`, which can only be used on {}.\n{} can have the following attributes: {}",
                s, similar, other, context, valid
            );
            return Err(syn::Error::new_spanned(src, msg)); // checked in tests/fail/derive_struct_attributes.rs
        }
    }

    let msg = format!("unknown attribute `{}`. Valid attributes are: {}", s, valid);
    Err(syn::Error::new_spanned(src, msg)) // checked in tests/fail/derive_struct_attributes.rs
}

pub struct Attribute<A: Attr> {
    kind: A,
    value: Option<syn::Expr>,
    src: TokenStream,
}

impl<A: Attr> Attribute<A> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut src = TokenStream::new();

        if input.peek(syn::LitStr) {
            let lit = input.parse::<syn::LitStr>()?;
            let value = syn::parse2::<syn::Expr>(quote! { #lit }).unwrap();
            src.extend(quote! { #value });

            let kind_name = if StrLit::new(lit).is_raw() {
                attr::All::FormatUnescaped.as_str()
            } else {
                attr::All::Format.as_str()
            };
            if let Some(&kind) = A::all().iter().find(|attr| attr.as_str() == kind_name) {
                return Ok(Self {
                    kind,
                    value: Some(value),
                    src,
                });
            }
            let name = attr::All::Format.as_str();
            let name2 = attr::All::FormatUnescaped.as_str();

            let valid = Context::ALL
                .iter()
                .filter(|c| c.all_attr_names().iter().any(|n| *n == name || *n == name2))
                .collect::<Vec<_>>();
            let valid = list_items(&valid, |c| c.to_string());

            let msg = format!(
                "omitting the attribute name is only valid for the `{}` attribute on {}",
                name, valid,
            );
            return Err(syn::Error::new_spanned(value, msg)); // checked in tests/fail/derive_field_attributes.rs
        }
        let attr = input.parse::<syn::Ident>()?;
        src.extend(quote! { #attr });
        let kind = find_match(&attr.to_string(), &src)?;

        let mut value = None;
        let peek = input.lookahead1();
        if !input.is_empty() && !peek.peek(Token![,]) {
            if !peek.peek(Token![=]) {
                return Err(peek.error()); // checked in tests/fail/derive_struct_attributes.rs
            }
            let eq_sign = input.parse::<Token![=]>()?;

            if input.is_empty() {
                let msg = "expected an expression after `=`";
                return Err(syn::Error::new_spanned(eq_sign, msg)); // checked in tests/fail/derive_struct_attributes.rs
            }
            let expr = input.parse::<syn::Expr>()?;
            src.extend(quote! { #eq_sign #expr });
            value = Some(expr);
        }

        Ok(Self { kind, value, src })
    }

    fn value_as<T: Parse>(&self, description: &str, addition: Option<&str>) -> Result<T> {
        if let Some(value) = &self.value {
            Ok(syn::parse2(quote! { #value })?)
        } else {
            let mut msg = format!(
                "attribute `{0}` has the format: `#[sscanf({0} = {1})]`",
                self.kind, description
            );
            if let Some(addition) = addition {
                msg.push('\n');
                msg.push_str(addition);
            }
            Error::err_spanned(&self.src, msg)
        }
    }
}

fn find_attrs<A: Attr>(attrs: Vec<syn::Attribute>) -> Result<HashMap<A, Attribute<A>>> {
    let mut ret = HashMap::<A, Attribute<A>>::new();
    for attr in attrs {
        if !attr.path().is_ident("sscanf") {
            continue;
        }
        let attr = match attr.meta {
            syn::Meta::List(l) => l,
            // the below _could_ be done simpler with syn::Meta::require_list, but the error
            // message in the `NameValue` case would just be "expected a '('" with a span
            // underlining the '=' sign, which is not very helpful
            syn::Meta::Path(p) => {
                let msg = "expected attribute arguments in parentheses: `sscanf(...)`";
                return Error::err_spanned(p, msg); // checked in tests/fail/derive_struct_attributes.rs
            }
            syn::Meta::NameValue(nv) => {
                let msg = format!(
                    "attribute arguments must be in parentheses: `sscanf({})`",
                    nv.value.to_token_stream()
                );
                return Error::err_spanned(nv, msg); // checked in tests/fail/derive_struct_attributes.rs
            }
        };

        let args = attr.parse_args::<TokenStream>()?;
        if args.is_empty() {
            // trying to parse empty args the regular way would give
            // the Parse implementation no tokens to point an error to
            // => check for empty args here
            continue;
        }
        let parsed = attr.parse_args_with(attr_parser::<A>())?;
        for attr in parsed {
            use std::collections::hash_map::Entry;
            match ret.entry(attr.kind) {
                Entry::Occupied(entry) => {
                    let msg = format!("attribute `{}` is specified multiple times", attr.kind);
                    return Error::builder()
                        .with_spanned(attr.src, msg)
                        .with_spanned(&entry.get().src, "previous use here")
                        .build_err(); // checked in tests/fail/derive_struct_attributes.rs
                }
                Entry::Vacant(entry) => {
                    entry.insert(attr);
                }
            }
        }
    }

    Ok(ret)
}

fn expect_one<A: Attr>(attrs: HashMap<A, Attribute<A>>) -> Result<Option<Attribute<A>>> {
    let mut attrs = attrs.into_values().collect::<Vec<_>>();
    attrs.sort_by_key(|attr| attr.kind); // so that the error messages always have the same order

    match attrs.len() {
        0 | 1 => Ok(attrs.into_iter().next()),
        2 => {
            let msg = format!(
                "cannot specify both `{}` and `{}`",
                attrs[0].kind, attrs[1].kind
            );
            Error::builder()
                .with_spanned(&attrs[0].src, &msg)
                .with_spanned(&attrs[1].src, &msg)
                .build_err() // checked in tests/fail/derive_struct_attributes.rs
        }
        _ => {
            let items = list_items(&attrs, |attr| format!("`{}`", attr.kind));
            let msg = format!("only one of {} is allowed", items);
            let mut error = Error::builder();
            for attr in attrs {
                error.with_spanned(attr.src, &msg);
            }
            error.build_err() // checked in tests/fail/derive_struct_attributes.rs
        }
    }
}

pub trait FromAttribute<A: Attr, Data = ()> {
    fn from_attribute(attr: Attribute<A>, data: Data) -> Result<Self>
    where
        Self: Sized;
}

pub struct SingleAttributeContainer<A: Attr, Kind, Data = ()>
where
    Kind: FromAttribute<A, Data>,
{
    pub src: TokenStream,
    pub kind: Kind,
    _marker: std::marker::PhantomData<(A, Data)>,
}

impl<A: Attr, Kind> SingleAttributeContainer<A, Kind>
where
    Kind: FromAttribute<A>,
{
    pub fn from_attrs(attrs: Vec<syn::Attribute>) -> Result<Option<Self>> {
        Self::from_attrs_with(attrs, ())
    }
}

impl<A: Attr, Kind, Data> SingleAttributeContainer<A, Kind, Data>
where
    Kind: FromAttribute<A, Data>,
{
    pub fn new(src: TokenStream, kind: Kind) -> Self {
        Self {
            src,
            kind,
            _marker: std::marker::PhantomData,
        }
    }
    pub fn from_attrs_with(attrs: Vec<syn::Attribute>, data: Data) -> Result<Option<Self>> {
        let attrs = find_attrs::<A>(attrs)?;

        let attr = match expect_one(attrs)? {
            Some(attr) => attr,
            None => return Ok(None),
        };

        let src = attr.src.clone();
        let kind = Kind::from_attribute(attr, data)?;

        Ok(Some(Self::new(src, kind)))
    }
}
