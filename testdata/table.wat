(module
  (func $f0 (result i32) i32.const 0x42)
  (func $f1 (result i32) i32.const 0x43)
  (func $f2 (result i32) i32.const 0x44)
  (func $f3 (result i32) i32.const 0x45)
  (table $t funcref (elem $f0 $f1 $f2 $f3))
  (elem funcref (ref.func $f0) (item ref.func $f1) (item (ref.null func)) (ref.func $f3))
  (elem  (offset i32.const 2) funcref (ref.func $f2))
  (elem  (i32.const 2) funcref (ref.func $f2))
)
  (table funcref (elem $f0 $f1 $f2 $f3))
  (table funcref (elem $f3 $f2 $f1 $f0))
  (func (export "test") (param i32) (result i32)
        local.get 0
        call_indirect (result i32)
        ))


             
