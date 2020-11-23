#![allow(warnings)]

use ::proc_macro_lib::attr;

const _: () = {
    #[attr]
    fn foo (_: (), second: (), arg_0: (), (x, ): (i32, ))
    {
        println!("foo")
    }

    #[attr]
    async fn bar ()
    {}

    #[attr]
    fn baz<T>()
    {}

    fn with_lifetime<'explicit>()
    {}
};

const _: () = {
    #[attr]
    trait Trait<T> : Sized
    where
        () : Copy,
    {
        fn foo (self, second: (), arg_0: (), (x, ): (i32, ))
        {
            println!("foo")
        }

        fn with_lifetime<'explicit>()
        {}
    }

    #[attr]
    impl<T, U> Trait<U> for Option<T>
    where
        () : Copy,
    {
        fn foo (self, second: (), arg_0: (), (x, ): (i32, ))
        {
            println!("foo")
        }

        fn with_lifetime<'explicit>()
        {}
    }
};

const _: () = {
    struct Foo<T>(T);

    #[attr]
    impl<T> Foo<T>
    where
        () : Copy,
    {
        fn foo (self: Self, second: (), arg_0: (), (x, ): (i32, ))
        {
            println!("foo")
        }

        fn with_lifetime<'explicit>()
        {}
    }
};
