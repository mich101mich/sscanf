use crate::*;

use std::collections::HashSet;

pub fn extract_lifetimes(ty: &syn::Type, out: &mut HashSet<syn::Lifetime>) {
    let mut lifetimes = LifetimeSet::new(out);
    extract_type_lifetimes(ty, &mut lifetimes);
}
struct LifetimeSet<'l> {
    lifetimes: &'l mut HashSet<syn::Lifetime>,
    excludes: HashSet<syn::Lifetime>,
}
impl<'l> LifetimeSet<'l> {
    pub fn new(lifetimes: &'l mut HashSet<syn::Lifetime>) -> Self {
        Self {
            lifetimes,
            excludes: HashSet::new(),
        }
    }

    /// Add a lifetime to the set, if it is not in the excludes set.
    pub fn add(&mut self, lifetime: &syn::Lifetime) {
        if !self.excludes.contains(lifetime) {
            self.lifetimes.insert(lifetime.clone());
        }
    }

    /// Process a closure with some additional excludes.
    pub fn with_excludes(
        &mut self,
        added_excludes: &Option<syn::BoundLifetimes>,
        code: impl FnOnce(&mut Self),
    ) {
        let mut exclude_backup = None;
        if let Some(excludes) = added_excludes {
            exclude_backup = Some(self.excludes.clone());
            for exclude in &excludes.lifetimes {
                if let syn::GenericParam::Lifetime(lifetime) = exclude {
                    self.excludes.insert(lifetime.lifetime.clone());
                }
            }
        }

        code(self);

        if let Some(excludes) = exclude_backup {
            self.excludes = excludes;
        }
    }
}

fn extract_type_lifetimes(ty: &syn::Type, out: &mut LifetimeSet) {
    match ty {
        syn::Type::Group(type_group) => extract_type_lifetimes(&type_group.elem, out),
        syn::Type::Paren(type_paren) => extract_type_lifetimes(&type_paren.elem, out),
        syn::Type::Array(type_array) => extract_type_lifetimes(&type_array.elem, out),
        syn::Type::Path(type_path) => {
            if let Some(qself) = &type_path.qself {
                extract_type_lifetimes(&qself.ty, out);
            }
            extract_path_lifetimes(&type_path.path, out)
        }
        syn::Type::Ptr(type_ptr) => extract_type_lifetimes(&type_ptr.elem, out),
        syn::Type::Reference(type_reference) => {
            if let Some(lifetime) = &type_reference.lifetime {
                out.add(lifetime);
            }
            extract_type_lifetimes(&type_reference.elem, out);
        }
        syn::Type::Slice(type_slice) => extract_type_lifetimes(&type_slice.elem, out),
        syn::Type::Tuple(type_tuple) => {
            for elem in &type_tuple.elems {
                extract_type_lifetimes(elem, out);
            }
        }
        syn::Type::TraitObject(type_trait_object) => {
            for bound in &type_trait_object.bounds {
                extract_type_param_bound_lifetimes(bound, out);
            }
        }
        syn::Type::ImplTrait(type_impl_trait) => {
            for bound in &type_impl_trait.bounds {
                extract_type_param_bound_lifetimes(bound, out);
            }
        }
        syn::Type::BareFn(type_bare_fn) => {
            out.with_excludes(&type_bare_fn.lifetimes, |out| {
                for param in &type_bare_fn.inputs {
                    extract_type_lifetimes(&param.ty, out);
                }
                if let syn::ReturnType::Type(_, output) = &type_bare_fn.output {
                    extract_type_lifetimes(output, out);
                }
            });
        }

        syn::Type::Infer(_) => {}    // doesn't have lifetimes
        syn::Type::Never(_) => {}    // doesn't have lifetimes
        syn::Type::Macro(_) => {}    // with a macro, we can't know which lifetimes will be used
        syn::Type::Verbatim(_) => {} // if syn doesn't know what it is, we can't either
        _ => {}
    }
}

fn extract_path_lifetimes(ty: &syn::Path, out: &mut LifetimeSet) {
    for segment in &ty.segments {
        match &segment.arguments {
            syn::PathArguments::None => {} // no lifetimes in this segment
            syn::PathArguments::AngleBracketed(angle_bracketed_generic_arguments) => {
                extract_angle_bracketed_lifetimes(angle_bracketed_generic_arguments, out);
            }
            syn::PathArguments::Parenthesized(parenthesized_generic_arguments) => {
                for arg in &parenthesized_generic_arguments.inputs {
                    extract_type_lifetimes(arg, out);
                }
                if let syn::ReturnType::Type(_, output) = &parenthesized_generic_arguments.output {
                    extract_type_lifetimes(output, out);
                }
            }
        }
    }
}

fn extract_angle_bracketed_lifetimes(
    args: &syn::AngleBracketedGenericArguments,
    out: &mut LifetimeSet,
) {
    for arg in &args.args {
        match arg {
            syn::GenericArgument::Lifetime(lifetime) => {
                out.add(lifetime);
            }
            syn::GenericArgument::Type(ty) => {
                extract_type_lifetimes(ty, out);
            }
            syn::GenericArgument::Const(_) => {} // constants don't have lifetimes
            syn::GenericArgument::AssocType(assoc_type) => {
                if let Some(angle_bracketed) = &assoc_type.generics {
                    extract_angle_bracketed_lifetimes(angle_bracketed, out);
                }
                extract_type_lifetimes(&assoc_type.ty, out);
            }
            syn::GenericArgument::AssocConst(_) => {} // constants don't have lifetimes
            syn::GenericArgument::Constraint(constraint) => {
                if let Some(angle_bracketed) = &constraint.generics {
                    extract_angle_bracketed_lifetimes(angle_bracketed, out);
                }
                for bound in &constraint.bounds {
                    extract_type_param_bound_lifetimes(bound, out);
                }
            }
            _ => {}
        }
    }
}

fn extract_type_param_bound_lifetimes(bound: &syn::TypeParamBound, out: &mut LifetimeSet) {
    match bound {
        syn::TypeParamBound::Trait(trait_bound) => {
            out.with_excludes(&trait_bound.lifetimes, |out| {
                extract_path_lifetimes(&trait_bound.path, out);
            });
        }
        syn::TypeParamBound::Lifetime(lifetime) => {
            out.add(lifetime);
        }
        syn::TypeParamBound::PreciseCapture(precise_capture) => {
            for param in &precise_capture.params {
                if let syn::CapturedParam::Lifetime(lifetime) = param {
                    out.add(lifetime);
                }
            }
        }
        syn::TypeParamBound::Verbatim(token_stream) => {} // if syn doesn't know what it is, we can't either
        _ => {}
    }
}
