# Unlisp

Interpreter for a toy Lisp language.

## Features (in a random order)

### Literals

```
>>> 1
1
>>> "foo"
"foo"
>>> (+ 1 2)
3
```

### Basic Lisp special forms

```
>>> (if t 1 2)
1
>>> (if nil 1 2)
2
>>> (let ((x 1) (y x)) (+ x y))
2
```

### Lists

```
>>> (cons 1 nil)
(1)
>>> (rest (list 1 2))
(2)
>>> (first (list 1 2))
1
```

### Lisp-2 peculiarities

```
>>> (defun foo (x y) (+ x y))
nil
>>> (foo 1 2)
3

;; this call raises an error because
;; functions are "stored" in a different namespace
;; and need to be accessed in a special way
>>> (let ((func foo)) (func 1 2))
error: undefined symbol foo
stack trace:
  <top>

;; this call raises an error because
;; function objects need to be called using funcall
>>> (let ((func (symbol-function (quote foo)))) (func 1 2))
error: undefined function func
stack trace:
  <top>

>>> (let ((func (symbol-function (quote foo)))) (funcall func 1 2))
3
```

### Lambdas and higher-order functions

```
>>> (funcall (lambda (f x) (funcall f (funcall f (funcall f x)))) (symbol-function (quote list)) nil)
(((nil)))
```

### Varargs

```
>>> (defun list (& args) args)
nil
>>> (list 1 2 3)
(1 2 3)
```

### Apply

```
>>> (apply (symbol-function (quote +)) (list 1 2))
3
>>> (apply (symbol-function (quote +)) 1 2 (list 3 4))
10
```

### "Standard library"

It is located in file [`src/stdlib.unl`](https://github.com/OlegTheCat/unlisp/blob/master/src/stdlib.unl).

### Macros & quasiquote

Quasiquote [is implemented](https://github.com/olegthecat/unlisp/blob/67e09b67905d6f9129eed04c0b1540d3bd55212d/src/stdlib.unl#L54-L112) using Unlisp's macro system. There are three macros, namely `qquote` which is quasiquote (like a backtick in other popular lisps), `unq` which stands for "unquote", and `unqs` which stands for "unquote-splicing".

```
>>> (defmacro strange-let (bindings & body)
  (reduce
   (lambda (acc binding)
     (let ((sym (first binding))
           (val (first (rest binding))))
       (qquote
        (funcall
         (lambda ((unq sym))
           (unq acc))
         (unq val)))))
   (qquote (let () (unqs body)))
   (reverse bindings)))
nil
>>> (strange-let ((x 1) (y 2) (z 3)) (+ x y z))
6
>>> (macroexpand-1 (quote (strange-let ((x 1) (y 2) (z 3)) (+ x y z))))
(funcall (lambda (x) (funcall (lambda (y) (funcall (lambda (z) (let nil (+ x y z))) 3)) 2)) 1)
```

### Printing and writing to stdout

```
>>> (print 1)
11
>>> (println 1)
1
1
>>> (println "foo")
"foo"
"foo"
>>> (stdout-write "foo")
foonil
```

### Error reporting

```
>>> (let foo)
error: let bindings are not a list
stack trace:
  <top>

>>> (- "foo" "bar")
error: cannot cast "foo" to i64
stack trace:
  -
  <top>

>>> (mapcar (symf (quote +)))
error: wrong number of arguments (1) passed to mapcar
stack trace:
  <top>
```

### Stacktraces

```
>>> (mapcar (symbol-function (quote +)) (list 1 2 3) (list 1 2 (quote x)))
error: cannot cast x to i64
stack trace:
  lambda/+/0+
  apply
  lambda/mapcar/2+
  apply
  lambda/mapcar/2+
  apply
  mapcar
  <top>
```
