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
    Result,
};

use ::core::ops::Not as _;

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
        call_site_args: func.inputs.iter_mut().enumerate().map(|(n, fn_arg)| Some(match *fn_arg {
            | FnArg::Receiver(ref receiver) => {
                let self_ =
                    format_ident!("self", span = receiver.self_token.span)
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
        })).collect::<Option<Vec<Expr>>>()?,
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
                    __Self: ?Sized + #trait_name
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
