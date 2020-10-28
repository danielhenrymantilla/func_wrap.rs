use ::func_wrap::*;
use ::proc_macro2::TokenStream;
use ::quote::ToTokens;
use ::syn::*;

macro_rules! parse_macro_input_2 {( $input:expr ) => (
    match ::syn::parse2($input) {
        | Ok(it) => it,
        | Err(err) => panic!("{}", err),
    }
)}

#[cfg(any())]
macro_rules! dbg_parse_quote {( $($tt:tt)* ) => (
    {
        eprintln!("{}", ::quote::quote!( $($tt)* ));
        parse_quote!( $($tt)* )
    }
)}

macro_rules! ts {( $($code:tt)* ) => (
    stringify!( $($code)* )
        .parse::<TokenStream>()
        .unwrap()
)}

fn assert_token_stream_eq (
    ts1: TokenStream,
    ts2: TokenStream,
)
{
    fn assert_tt_eq (
        tt1: ::proc_macro2::TokenTree,
        tt2: ::proc_macro2::TokenTree,
    )
    {
        use ::proc_macro2::TokenTree::*;
        match (tt1, tt2) {
            | (Group(g1), Group(g2)) => assert_token_stream_eq(g1.stream(), g2.stream()),
            | (Ident(lhs), Ident(rhs)) => assert_eq!(lhs.to_string(), rhs.to_string()),
            | (Punct(lhs), Punct(rhs)) => assert_eq!(lhs.as_char(), rhs.as_char()),
            | (Literal(lhs), Literal(rhs)) => assert_eq!(lhs.to_string(), rhs.to_string()),
            | _ => panic!("Not equal!"),
        }
    }

    let mut ts1 = ts1.into_iter();
    let mut ts2 = ts2.into_iter();
    loop {
        match (ts1.next(), ts2.next()) {
            | (Some(tt1), Some(tt2)) => assert_tt_eq(tt1, tt2),
            | (None, None) => return,
            | _ => panic!("Not equal!"),
        }
    }
}

#[test]
fn simple_fn ()
{
    fn proc_macro (input: TokenStream) -> TokenStream
    {
        let mut input: ItemFn = parse_macro_input_2!(input);
        let wrapped_func_call = func_wrap(
            &mut input.sig,
            ::core::mem::replace(&mut input.block, parse_quote!( {} )),
            None,
        ).unwrap();
        input.block = parse_quote!(
            { #wrapped_func_call }
        );
        input.into_token_stream()
    }

    let input = ts! {
        fn foo (_: (), second: (), arg_0: (), (x, ): (i32, ))
        {
            println!("foo")
        }
    };
    assert_token_stream_eq(proc_macro(input), ts! {
        fn foo (arg_0: (), second: (), arg_2: (), arg_3: (i32, ))
        {
            ({
                #[inline(always)]
                fn foo (_: (), second: (), arg_0: (), (x, ): (i32, ))
                {
                    println!("foo")
                }

                foo
            })(arg_0, second, arg_2, arg_3)
        }
    });
}

#[test]
fn default_method ()
{
    fn proc_macro (input: TokenStream) -> TokenStream
    {
        let mut input: ItemTrait = parse_macro_input_2!(input);
        let method =
            if let TraitItem::Method(ref mut it) = input.items[0] {
                it
            } else {
                panic!()
            }
        ;
        let method_block = if let Some(ref mut it) = method.default { it } else {
            panic!()
        };
        let outer_scope = (
            &input.generics,
            ImplOrTrait::DefaultMethod { trait_name: &input.ident },
        );
        let wrapped_func_call = func_wrap(
            &mut method.sig,
            ::core::mem::replace(method_block, parse_quote!( {} )),
            Some(outer_scope),
        ).unwrap();
        *method_block = parse_quote!(
            { #wrapped_func_call }
        );
        input.into_token_stream()
    }

    let input = ts! {
        trait Foo<T>
        where
            () : Copy,
        {
            fn foo (self, second: (), arg_0: (), (x, ): (i32, ))
            {
                println!("foo")
            }
        }
    };
    assert_token_stream_eq(proc_macro(input), ts! {
        trait Foo<T>
        where
            () : Copy,
        {
            fn foo (self, second: (), arg_2: (), arg_3: (i32, ))
            {
                ({
                    trait __FuncWrap<T> : Foo<T>
                    where
                        () : Copy,
                    {
                        #[inline(always)]
                        fn foo (self, second: (), arg_0: (), (x, ): (i32, ))
                        {
                            println!("foo")
                        }
                    }

                    impl<T, __Self : ?Sized + Foo<T>> __FuncWrap<T>
                        for __Self
                    where
                        () : Copy,
                    {}

                    <Self as __FuncWrap<T>>::foo
                })(self, second, arg_2, arg_3)
            }
        }
    });
}

mod impls {
    use super::*;

    fn proc_macro (input: TokenStream) -> TokenStream
    {
        let mut input: ItemImpl = parse_macro_input_2!(input);
        let method =
            if let ImplItem::Method(ref mut it) = input.items[0] {
                it
            } else {
                panic!()
            }
        ;
        let outer_scope = (
            &input.generics,
            ImplOrTrait::ImplMethod {
                implementor: &input.self_ty,
                trait_name: input.trait_.as_ref().map(|(_, it, _)| it)
            },
        );
        let wrapped_func_call = func_wrap(
            &mut method.sig,
            ::core::mem::replace(&mut method.block, parse_quote!( {} )),
            Some(outer_scope),
        ).unwrap();
        method.block = parse_quote!(
            { #wrapped_func_call }
        );
        input.into_token_stream()
    }

    #[test]
    fn inherent ()
    {
        let input = ts! {
            impl<T> Foo<T>
            where
                () : Copy,
            {
                fn foo (self: Self, second: (), arg_0: (), (x, ): (i32, ))
                {
                    println!("foo")
                }
            }
        };
        assert_token_stream_eq(proc_macro(input), ts! {
            impl<T> Foo<T>
            where
                () : Copy,
            {
                fn foo (self: Self, second: (), arg_2: (), arg_3: (i32, ))
                {
                    ({
                        trait __FuncWrap<T>
                        where
                            () : Copy,
                        {
                            fn foo (self: Self, _: (), _: (), _: (i32, ))
                            ;
                        }

                        impl<T> __FuncWrap<T>
                            for Foo<T>
                        where
                            () : Copy,
                        {
                            #[inline(always)]
                            fn foo (self: Self, second: (), arg_0: (), (x, ): (i32, ))
                            {
                                println!("foo")
                            }
                        }

                        <Self as __FuncWrap<T>>::foo
                    })(self, second, arg_2, arg_3)
                }
            }
        });
    }

    #[test]
    fn trait_for ()
    {
        let input = ts! {
            impl<T, U> Trait<U> for Foo<T>
            where
                () : Copy,
            {
                fn foo (self, second: (), arg_0: (), (x, ): (i32, ))
                {
                    println!("foo")
                }
            }
        };
        assert_token_stream_eq(proc_macro(input), ts! {
            impl<T, U> Trait<U> for Foo<T>
            where
                () : Copy,
            {
                fn foo (self, second: (), arg_2: (), arg_3: (i32, ))
                {
                    ({
                        trait __FuncWrap<T, U> : Trait<U>
                        where
                            () : Copy,
                        {
                            fn foo (self, _: (), _: (), _: (i32, ))
                            ;
                        }

                        impl<T, U> __FuncWrap<T, U>
                            for Foo<T>
                        where
                            () : Copy,
                        {
                            #[inline(always)]
                            fn foo (self, second: (), arg_0: (), (x, ): (i32, ))
                            {
                                println!("foo")
                            }
                        }

                        <Self as __FuncWrap<T, U>>::foo
                    })(self, second, arg_2, arg_3)
                }
            }
        });
    }
}
