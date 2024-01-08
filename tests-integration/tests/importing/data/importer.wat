(module 
  (type $void (func))
  (type $f1 (func (param i32) (result i32)))
  (import "src" "f2" (func $f (type 1)))
  (func (export "test") (param i32) (result i32) 
        local.get 0
        call $f
        i32.const 5
        i32.add
    )
  )
