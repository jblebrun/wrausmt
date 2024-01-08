(module)

(assert_invalid
    (module (func (result i32) (i32.const 1) (i32.add)))
    "type mismatch"
)

(assert_invalid
    (module (func (result i32) (i32.const 1) (i32.const 1) (i64.add)))
    "type mismatch"
)

(assert_invalid
    (module (func (result i32) (i64.const 1) (i32.const 1) (i64.add)))
    "type mismatch"
)

(assert_invalid
    (module (func (result i32) (i32.const 1) (i64.const 1) (i64.add)))
    "type mismatch"
)