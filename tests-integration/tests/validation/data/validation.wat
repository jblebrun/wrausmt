(module (func (result i32) (i32.const 2) (i32.const 1) (i32.add)))
(module (func (result i64) (i64.const 2) (i64.const 1) (i64.add)))
(module (func (result f32) (f32.const 2) (f32.const 1) (f32.add)))
(module (func (result f64) (f64.const 2) (f64.const 1) (f64.add)))

(module (func (export "param") (param i32) (result i32) (local.get 0) (i32.const 42) (i32.add)))
(assert_return (invoke "param" (i32.const 1)) (i32.const 43))

(module (func (export "local") (result i32) (local i32) (i32.const 1) (local.set 0) (local.get 0) (i32.const 42) (i32.add)))
(assert_return (invoke "local") (i32.const 43))

(module (func (export "local-tee") (result i32) (local i32) (i32.const 1) (local.tee 0) (i32.const 42) (i32.add)))
(assert_return (invoke "local-tee") (i32.const 43))

(module (func (result i32) (block (result i32) (i32.const 42))))

(module 
 (func (export "simple-if") (param i32) (result i32)
    (if (result i32) (local.get 0) (then (i32.const 7)) (else (i32.const 8))))
)
(assert_return (invoke "simple-if" (i32.const 0)) (i32.const 8))
(assert_return (invoke "simple-if" (i32.const 1)) (i32.const 7))

(module (func (export "type-i32-value") (result i32)
    (block (result i32) (i32.ctz (br 0 (i32.const 1))))
))
(assert_return (invoke "type-i32-value") (i32.const 1))

(module (func (export "as-loop-first") (result i32)
    (block (result i32) (loop (result i32) (br 1 (i32.const 3)) (i32.const 2)))
))

(assert_return (invoke "as-loop-first") (i32.const 3))

(module (func (export "type-i32-value") (result i32)
    (block (result i32) (i32.ctz (br_if 0 (i32.const 1) (i32.const 1))))
))

  (func (export "singleton") (param i32) (result i32)
    (block
      (block
        (br_table 1 0 (local.get 0))
        (return (i32.const 21))
      )
      (return (i32.const 20))
    )
    (i32.const 22)
  )