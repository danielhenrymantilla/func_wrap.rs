extern crate proc_macro;
use ::proc_macro::TokenStream;
use ::quote::ToTokens;

use ::func_wrap::parse_and_func_wrap_with;

#[proc_macro_attribute] pub
fn attr (
    attrs: TokenStream,
    input: TokenStream,
) -> TokenStream
{
    ::syn::parse_macro_input!(attrs as ::syn::parse::Nothing);
    parse_and_func_wrap_with(input, |func, wrapped_func_call| Ok({
        if let Some(wrapped_func_call) = wrapped_func_call {
            func.block = ::syn::parse_quote!({ #wrapped_func_call });
        } else {
            return Err(::syn::Error::new(
                ::proc_macro2::Span::call_site(),
                "Missing `#[attr]` on the enscoping `trait` or `impl`",
            ));
        }
    }))
    .map_or_else(|err| err.to_compile_error(), |it| it.into_token_stream())
    .into()
}
