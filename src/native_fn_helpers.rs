#![macro_use]

macro_rules! define_native_fn {
    ($id:ident ($env:ident, $( $arg:ident : $converter:path ),*) -> $result_wrap:path $body:block) => {
        #[allow(unused_mut)]
        fn $id( $env: &mut core::Env, lo: LispObject ) -> error::GenResult<LispObject> {
            let mut form = core::to_vector(lo)?;
            let mut args = form.slice(1..);

            let mut parameters_count = 0;
            $( stringify!($arg); parameters_count += 1; )*

                if parameters_count != args.len() {
                    return Err(Box::new(
                        error::ArityError::new(parameters_count,
                                               args.len(),
                                               stringify!($id).to_string())));
                }

            let mut iter = args.into_iter();
            $( let mut $arg = $converter(iter.next().unwrap())?; )*

            let res = $result_wrap($body);
            Ok(res)
        }
    };

    ($id:ident ($env:ident, $( $arg:ident : $converter:path, )* ... $vararg:ident : $vconverter:path ) -> $result_wrap:path $body:block) => {
        #[allow(unused_mut)]
        fn $id( $env: &mut core::Env, lo: LispObject ) -> error::GenResult<LispObject> {
            let mut form = core::to_vector(lo)?;
            let mut args = form.slice(1..);

            #[allow(unused_mut)]
            let mut non_vararg_parameters_count = 0;
            $( stringify!($arg); non_vararg_parameters_count += 1; )*

                if non_vararg_parameters_count > args.len() {
                    return Err(Box::new(
                        error::ArityError::new(non_vararg_parameters_count,
                                               args.len(),
                                               stringify!($id).to_string())));
                }

            #[allow(unused_mut)]
            let mut iter = args.into_iter();

            $( #[allow(unused_mut)] let mut $arg = $converter(iter.next().unwrap())?; )*

            let mut $vararg: Vector<_> = iter
                .map(|lo| $vconverter(lo))
                .collect::<Result<Vector<_>, _>>()?;

            let res = $result_wrap($body);
            Ok(res)
        }
    }
}
