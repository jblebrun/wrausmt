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
