use crate::*;

#[derive(Clone)]
pub struct Type<'a> {
    pub kind: TypeKind,
    pub source: TypeSource<'a>,
    ty: syn::Type,
}

#[derive(Clone)]
pub enum TypeKind {
    Str(Option<syn::Lifetime>),
    CowStr(Option<syn::Lifetime>),
    Other,
}

#[derive(Clone)]
pub enum TypeSource<'a> {
    External,
    Format(StrLitSlice<'a>),
}

#[allow(unused)]
impl<'a> Type<'a> {
    pub fn from_ty(ty: syn::Type) -> Self {
        let kind = TypeKind::from_ty(&ty);
        let source = TypeSource::External;
        Type { kind, source, ty }
    }
    pub fn inner(&self) -> &syn::Type {
        &self.ty
    }
    pub fn into_inner(self) -> syn::Type {
        self.ty
    }
    pub fn full_span(&self) -> FullSpan {
        match &self.source {
            TypeSource::External => FullSpan::from_spanned(&self.ty),
            TypeSource::Format(src) => FullSpan::from_span(src.span()),
        }
    }

    pub fn lifetime(&self) -> Option<&syn::Lifetime> {
        self.kind.lifetime()
    }
    pub fn err<T, U: std::fmt::Display>(&self, message: U) -> Result<T> {
        Err(self.error(message))
    }
    pub fn error<U: std::fmt::Display>(&self, message: U) -> Error {
        match &self.source {
            TypeSource::External => Error::new_spanned(&self.ty, message),
            TypeSource::Format(src) => src.error(message),
        }
    }
    pub fn from_str(src: StrLitSlice<'a>) -> syn::Result<Self> {
        let span = src.span();

        let tokens = src.text().parse::<TokenStream>()?.with_span(span);
        let mut ty = syn::parse2::<Type>(tokens)?;
        ty.source = TypeSource::Format(src);
        Ok(ty)
    }
}

impl TypeKind {
    pub fn from_ty(src: &syn::Type) -> Self {
        if let Some(lt) = ty_check::get_str(src) {
            TypeKind::Str(lt)
        } else if let Some(lt) = ty_check::get_cow_str(src) {
            TypeKind::CowStr(lt)
        } else {
            TypeKind::Other
        }
    }
    pub fn lifetime(&self) -> Option<&syn::Lifetime> {
        match self {
            TypeKind::Str(lt) | TypeKind::CowStr(lt) => lt.as_ref(),
            TypeKind::Other => None,
        }
    }
}

impl Parse for Type<'_> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(Token![&]) {
            // possibly &str
            let ty: syn::Type = input.parse::<syn::TypeReference>()?.into();
            let ret = Self::from_ty(ty);
            if !matches!(ret.kind, TypeKind::Str(_)) {
                let msg = "references are only allowed for &str";
                return Err(syn::Error::new_spanned(ret, msg)); // TODO: check
            } else if !matches!(ret.kind, TypeKind::Str(None)) {
                let msg = "str references are not allowed to have lifetimes. The lifetime of the return value is set to the lifetime of the input string";
                return Err(syn::Error::new_spanned(ret, msg)); // TODO: check
            }
            Ok(ret)
        } else {
            let ty: syn::Type = input.parse::<syn::TypePath>()?.into();
            Ok(Self::from_ty(ty))
        }
    }
}

impl quote::ToTokens for Type<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.ty.to_tokens(tokens);
    }
}

pub mod ty_check {
    use super::*;

    fn is_segment(segment: &syn::PathSegment, name: &str) -> bool {
        segment.ident == name && matches!(segment.arguments, syn::PathArguments::None)
    }

    pub fn is_str_path(ty: &syn::TypePath) -> bool {
        let possible = [
            quote! { str },
            quote! { primitive::str },
            quote! { std::primitive::str },
            quote! { ::std::primitive::str },
            quote! { core::primitive::str },
            quote! { ::core::primitive::str },
        ];
        let ty = ty.to_token_stream().to_string();
        possible.iter().any(|p| p.to_string() == ty)
    }
    pub fn get_str(ty: &syn::Type) -> Option<Option<syn::Lifetime>> {
        match ty {
            syn::Type::Path(ref ty) => {
                if is_str_path(ty) {
                    Some(None)
                } else {
                    None
                }
            }
            syn::Type::Reference(ref ty) => {
                if ty.mutability.is_none()
                    && matches!(*ty.elem, syn::Type::Path(ref inner) if is_str_path(inner))
                {
                    Some(ty.lifetime.clone())
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub(crate) fn get_cow_str_path(ty: &syn::TypePath) -> Option<Option<syn::Lifetime>> {
        if ty.qself.is_some() {
            return None;
        }

        type Iter<'a> = syn::punctuated::Iter<'a, syn::PathSegment>;
        fn get_cow_segment(mut iter: Iter) -> Option<Option<syn::Lifetime>> {
            let seg = iter.next()?;
            if iter.next().is_some() {
                return None;
            }
            if seg.ident != "Cow" {
                return None;
            }
            let mut args = match &seg.arguments {
                syn::PathArguments::AngleBracketed(args) => args.args.iter(),
                _ => return None,
            };
            let mut ty = args.next()?;
            let ret = if let syn::GenericArgument::Lifetime(lt) = ty {
                ty = args.next()?;
                Some(lt.clone())
            } else {
                None
            };
            if !matches!(ty, syn::GenericArgument::Type(syn::Type::Path(ref inner)) if is_str_path(inner))
            {
                return None;
            }
            if args.next().is_some() {
                return None;
            }
            Some(ret)
        }
        fn get_module_segment(mut iter: Iter) -> Option<Option<syn::Lifetime>> {
            if is_segment(iter.next()?, "borrow") {
                get_cow_segment(iter)
            } else {
                None
            }
        }
        fn get_root_segment(mut iter: Iter) -> Option<Option<syn::Lifetime>> {
            let seg = iter.next()?;
            if !is_segment(seg, "std") && !is_segment(seg, "alloc") {
                return None;
            }
            get_module_segment(iter)
        }

        let iter = ty.path.segments.iter();
        if ty.path.leading_colon.is_some() {
            get_root_segment(iter)
        } else {
            get_root_segment(iter.clone())
                .or_else(|| get_module_segment(iter.clone()))
                .or_else(|| get_cow_segment(iter))
        }
    }
    pub fn get_cow_str(ty: &syn::Type) -> Option<Option<syn::Lifetime>> {
        match ty {
            syn::Type::Path(ref ty) => get_cow_str_path(ty),
            _ => None,
        }
    }
}
