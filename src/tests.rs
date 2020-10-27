use super::*;

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

fn proc_macro (input: TokenStream)
  -> TokenStream
{
    parse_and_func_wrap_with(input, |func, wrapped_func_call| Ok({
        if let Some(wrapped_func_call) = wrapped_func_call {
            func.block = parse_quote!({ #wrapped_func_call });
        } else {
            return Err(Error::new(Span::call_site(),
                "Missing `#[proc_macro]` on the enscoping `trait` or `impl`",
            ));
        }
    })).map_or_else(|err| err.to_compile_error(), |it| it.into_token_stream())
}

#[test]
fn missing_enscoping ()
{
    let input = ts! {
        fn next (self: &'_ mut Self)
          -> Option<Self::Item>
        {}
    };

    assert_token_stream_eq(proc_macro(input), ts! {
        compile_error! {
            "Missing `#[proc_macro]` on the enscoping `trait` or `impl`"
        }
    });
}

#[test]
fn simple_fn ()
{
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
                    trait Helper<T> : Foo<T>
                    where
                        () : Copy,
                    {
                        #[inline(always)]
                        fn foo (self, second: (), arg_0: (), (x, ): (i32, ))
                        {
                            println!("foo")
                        }
                    }

                    impl<T, __Self : ?Sized + Foo<T>> Helper<T>
                        for __Self
                    where
                        () : Copy,
                    {}

                    <Self as Helper<T>>::foo
                })(self, second, arg_2, arg_3)
            }
        }
    });
}

mod impls {
    use super::*;

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
                        trait Helper<T>
                        where
                            () : Copy,
                        {
                            fn foo (self: Self, _: (), _: (), _: (i32, ))
                            ;
                        }

                        impl<T> Helper<T>
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

                        <Foo<T> as Helper<T>>::foo
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
                        trait Helper<T, U> : Trait<U>
                        where
                            () : Copy,
                        {
                            fn foo (self, _: (), _: (), _: (i32, ))
                            ;
                        }

                        impl<T, U> Helper<T, U>
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

                        <Foo<T> as Helper<T, U>>::foo
                    })(self, second, arg_2, arg_3)
                }
            }
        });
    }
}
