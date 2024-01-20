(module)

(assert_invalid 
    (module $missing-operand
        (func (result i32) (i32.const 1) (i32.add)))
    "type mismatch"
)

(assert_invalid 
    (module $wrong-operand-type-ab
        (func (result i32) (i32.const 1) (i32.const 1) (i64.add)))
    "type mismatch"
)

(assert_invalid 
    (module $wrong-operand-type-a
        (func (result i32) (i64.const 1) (i32.const 1) (i64.add)))
    "type mismatch"
)

(assert_invalid 
    (module $wrong-operand-type-b
        (func (result i32) (i32.const 1) (i64.const 1) (i64.add)))
    "type mismatch"
)

(assert_invalid 
    (module $set-wrong-local-type
        (func (local i64) (i32.const 1) (local.set 0)))
    "type mismatch"
)

(assert_invalid 
    (module $tee-wrong-local-type
        (func (result i64) (local i32) (local.tee 0)))
    "type mismatch"
)

(assert_invalid
    (module $wrong-local-type-as-param
        (func (result i32) (local i64) (local.get 0) (i32.const 1) (i32.add)))
    "type mismatch"
)

(assert_invalid
    (module $no-memory-to-init
        (data (i32.const 2) "\03\01\04\01")
        (data "\02\07\01\08")
        (func (memory.init 1 (i32.const 7) (i32.const 0) (i32.const 4))) 
    ) "unknown memory")

(assert_invalid
    (module $no-memory-to-init
        (data (i32.const 2) "\03\01\04\01")
        (data "\02\07\01\08")
        (func (memory.copy (i32.const 7) (i32.const 0) (i32.const 4))) 
    ) "unknown memory")

(assert_invalid
    (module $no-data-to-init
        (memory (export "memory0") 1 1)
        (data (i32.const 2) "\03\01\04\01")
        (func (memory.init 1 (i32.const 7) (i32.const 0) (i32.const 4))) 
    ) "unknown data")

(assert_invalid
  (func $f1 (export "funcref-not-i32") (param $x i32) (result i32)
    (ref.is_null (local.get $x))
  )
  "type mismatch"
)