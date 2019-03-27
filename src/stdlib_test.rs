use crate::test_utils::*;

fn ctx() -> Context {
    Context::new(true, true, true)
}

#[test]
fn test_quasiquote() {
    let ctx = ctx();

    assert_ok!(ctx, "(qquote 1)", "1");
    assert_ok!(ctx, "(qquote foo)", "foo");

    assert_ok!(ctx, "(qquote (unq 1))", "1");
    assert_ok!(ctx, "(qquote ((unqs (list 1 2 3))))", "(1 2 3)");

    assert_ok!(ctx, "(let ((x 1)) (qquote (unq x)))", "1");
    assert_ok!(
        ctx,
        "(let ((x (list 1 2 3))) (qquote ((unqs x))))",
        "(1 2 3)"
    );

    assert_ok!(ctx, "(let ((x 1)) (qquote (qquote (unq (unq x)))))", "1");
    assert_ok!(
        ctx,
        "(let ((x (quote foo))) (qquote (qquote (unq (unq x)))))",
        "foo"
    );

    assert_ok!(
        ctx,
        "(defmacro abbrev (long short)
           (qquote
            (defmacro (unq short) (& body)
              (qquote ((unq (quote (unq long))) (unqs body))))))

         (abbrev defun defn)
         (defn inc (x) (+ x 1))

         (inc 5)",
        "6"
    );
}
