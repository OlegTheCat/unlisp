#![macro_use]

macro_rules! define_vararg_native_fn {
    ($id:ident ($env:ident, $( $arg:ident : $converter:path, )* ... $vararg:ident : $vconverter:path ) -> $result_wrap:path $body:block) => {
        fn $id( $env: &mut core::Env, lo: LispObject ) -> LispObject {
            let mut form = core::to_vector(lo);
            let mut iter = form.slice(1..).into_iter();

            $( let mut $arg = $converter(iter.next().unwrap()); )*

            let $vararg: Vector<_> = iter
                .map(|lo| $vconverter(lo))
                .collect();

            $result_wrap($body)
        }
    }
}



macro_rules! define_native_fn {
    ($id:ident ($env:ident, $( $arg:ident : $converter:path ),* ) -> $result_wrap:path $body:block) => {
        fn $id( $env: &mut Env, lo: LispObject ) -> LispObject {
            let mut form = core::to_vector(lo);
            let args = form.slice(1..);
            let mut passed_args_count = 0;

            $( stringify!($arg); passed_args_count += 1; )*

            if passed_args_count != args.len() {
                panic!("Wrong number of args passed to {}", stringify!($id));
            }

            let mut iter = args.into_iter();

            $( let mut $arg = $converter(iter.next().unwrap()); )*
                $result_wrap($body)
        }
    }
}
