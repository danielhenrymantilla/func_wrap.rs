//! # `::func_wrap`
//!
//! Helper crate for procedural macro authors that wish to duplicate some
//! received function inside its body, so as to be able to _wrap_ with some
//! prologue, epilogue, cache-ing, _etc._
//!
//! ## Examples
//!
//! See [https://docs.rs/require_unsafe_in_body] for a real-life example of
//! using it.
//!
//! [https://docs.rs/require_unsafe_in_body]: https://docs.rs/require_unsafe_in_body

#![allow(nonstandard_style, unused_imports)]

use ::proc_macro2::{
    Span, TokenStream,
};

use ::quote::{
    format_ident, quote, quote_spanned, ToTokens,
};

use ::syn::{*,
    parse::{Parse, Parser, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    Result,
};

use ::core::{mem, ops::Not as _};

#[derive(Clone, Copy)]
pub
enum ImplOrTrait<'__> {
    /// Default implementation of methods within a trait definition.
    DefaultMethod {
        trait_name: &'__ Ident,
    },

    /// An implementation of methods within an `impl` block.
    ImplMethod {
        implementor: &'__ Type,
        trait_name: Option<&'__ Path>, // `None` if inherent impl.
    }
}
use ImplOrTrait::*;

pub
struct WrappedFuncCall<'__> {
    pub
    outer_scope: Option<(&'__ Generics, ImplOrTrait<'__>)>,

    pub
    sig: Signature,

    pub
    block: Block,

    pub
    call_site_args: Vec<Expr>,
}

pub
fn func_wrap<'lt> (
    func: &'_ mut Signature,
    block: Block,
    outer_scope: Option<(&'lt Generics, ImplOrTrait<'lt>)>,
) -> Option<WrappedFuncCall<'lt>>
{Some({
    WrappedFuncCall {
        sig: func.clone(),
        call_site_args:
            func.inputs
                .iter_mut()
                .enumerate()
                .map(|(n, fn_arg)| Some(match *fn_arg {
                    | FnArg::Receiver(ref receiver) => {
                        let self_ = format_ident!(
                            "self",
                            span = receiver.self_token.span,
                        )
                        ;
                        parse_quote!( #self_ )
                    },
                    | FnArg::Typed(ref mut pat_ty) => {
                        if let Pat::Ident(ref mut pat) = *pat_ty.pat {
                            let ident = &mut pat.ident;
                            if *ident == "self" {
                                if outer_scope.is_none() { return None; }
                            } else {
                                if ident.to_string().starts_with("arg_") {
                                    *ident = format_ident!("arg_{}", n);
                                }
                            }
                            parse_quote!( #ident )
                        } else {
                            let ident = format_ident!("arg_{}", n);
                            *pat_ty.pat = parse_quote!( #ident );
                            parse_quote!( #ident )
                        }
                    },
                }))
                .collect::<Option<Vec<Expr>>>()?
        ,
        outer_scope,
        block,
    }
})}

impl ToTokens for WrappedFuncCall<'_> {
    fn to_tokens (self: &'_ Self, out: &'_ mut TokenStream)
    {
        let Self { sig, outer_scope, block, call_site_args } = self;
        let fname = &sig.ident;
        out.extend(match outer_scope {
            | None => quote!(
                ({
                    #[inline(always)]
                    #sig
                    #block

                    #fname
                })(#(#call_site_args),*)
            ),

            | Some((
                generics,
                DefaultMethod { trait_name },
            )) => {
                let (intro_generics, feed_generics, where_clauses) =
                    generics.split_for_impl()
                ;
                let trait_def = quote!(
                    trait Helper #intro_generics
                    :
                        #trait_name #feed_generics
                    #where_clauses
                    {
                        #[inline(always)]
                        #sig
                        #block
                    }
                );
                let mut impl_generics = (*generics).clone();
                impl_generics.params.push(parse_quote!(
                    __Self: ?Sized + #trait_name #feed_generics
                ));
                let (impl_generics, _, _) = impl_generics.split_for_impl();
                quote!(
                    ({
                        #trait_def

                        impl #impl_generics
                            Helper #feed_generics
                        for
                            __Self
                        #where_clauses
                        {}

                        <Self as Helper #feed_generics>::#fname
                    })(#(#call_site_args),*)
                )
            },

            | Some((
                generics,
                ImplMethod { implementor, trait_name },
            )) => {
                let (intro_generics, feed_generics, where_clauses) =
                    generics.split_for_impl()
                ;
                let mut empty_sig = sig.clone();
                empty_sig.inputs.iter_mut().for_each(|fn_arg| match *fn_arg {
                    | FnArg::Typed(ref mut pat_ty)
                        if matches!(
                            *pat_ty.pat,
                            Pat::Ident(ref pat)
                            if pat.ident == "self"
                        ).not()
                    => {
                        *pat_ty.pat = parse_quote!( _ );
                    },
                    | _ => {},
                });
                let super_trait = trait_name.map(|it| quote!( : #it ));
                quote!(
                    ({
                        trait Helper #intro_generics
                            #super_trait
                        #where_clauses
                        {
                            #empty_sig;
                        }

                        impl #intro_generics
                            Helper #feed_generics
                        for
                            #implementor
                        #where_clauses
                        {
                            #[inline(always)]
                            #sig
                            #block
                        }

                        <#implementor as Helper #feed_generics>::#fname
                    })(#(#call_site_args),*)
                )
            },
        })
    }
}

pub
fn parse_and_func_wrap_with (
    input: impl Into<TokenStream>,
    mut with: impl FnMut(
        &'_ mut ImplItemMethod,
        Option<&'_ mut WrappedFuncCall<'_>>,
    ) -> Result<()>,
) -> Result<Item>
{Ok({
    let mut input: Item = parse2(input.into())?;
    match input {
        | Item::Fn(ref mut it_fn) => {
            let outer_scope = None;
            let ItemFn { attrs, vis, sig, block } =
                mem::replace(it_fn, parse_quote!( fn __() {} ))
            ;
            let mut func = ImplItemMethod {
                attrs, vis, sig,
                block: parse_quote!( {} ),
                defaultness: None,
            };
            let ref mut wrapped_func = func_wrap(
                &mut func.sig,
                *block,
                outer_scope,
            );
            let () = with(&mut func, wrapped_func.as_mut())?;
            let ImplItemMethod { attrs, vis, sig, block, .. } = func;
            *it_fn = ItemFn {
                attrs, vis, sig, block: Box::new(block),
            };
        },

        | Item::Trait(ref mut it_trait) => {
            let outer_scope = Some((
                &it_trait.generics,
                ImplOrTrait::DefaultMethod { trait_name: &it_trait.ident },
            ));

            it_trait.items.iter_mut().try_for_each(|it| Result::Ok(match *it {
                | TraitItem::Method(ref mut method) => match method.default {
                    | Some(ref mut block) => {
                        let block = mem::replace(block, parse_quote!( {} ));
                        let TraitItemMethod { attrs, sig, .. } =
                            mem::replace(method, parse_quote!(fn __ () {}))
                        ;
                        let mut func = ImplItemMethod {
                            attrs, sig,
                            vis: Visibility::Inherited,
                            block: parse_quote!( {} ),
                            defaultness: None,
                        };
                        let ref mut wrapped_func = func_wrap(
                            &mut func.sig,
                            block,
                            outer_scope,
                        );
                        let () = with(&mut func, wrapped_func.as_mut())?;
                        let ImplItemMethod { attrs, sig, block, .. } = func;
                        *method = TraitItemMethod {
                            attrs, sig, default: Some(block),
                            semi_token: None,
                        };
                    },
                    | _ => {},
                },
                | _ => {},
            }))?;
        },

        | Item::Impl(ref mut it_impl) => {
            let outer_scope = Some((
                &it_impl.generics,
                ImplOrTrait::ImplMethod {
                    implementor: &it_impl.self_ty,
                    trait_name: it_impl.trait_.as_ref().map(|(_, it, _)| it)
                },
            ));

            it_impl.items.iter_mut().try_for_each(|it| Result::Ok(match *it {
                | ImplItem::Method(ref mut func) => {
                    let ref mut wrapped_func = func_wrap(
                        &mut func.sig,
                        mem::replace(&mut func.block, parse_quote!( {} )),
                        outer_scope,
                    );
                    let () = with(func, wrapped_func.as_mut())?;
                },
                | _ => {},
            }))?;
        },

        | otherwise => return Err(Error::new(otherwise.span(),
            "Expected an `fn` item, a `trait` definition, or an `impl` block."
        )),
    }
    input
})}

#[cfg(test)]
mod tests;
