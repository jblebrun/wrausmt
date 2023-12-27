(module
  (func (export "f1") (param i32) (result i32)
        i32.const 10
        local.get 0
        i32.add
        )
  (func (export "f2") (param i32) (result i32)
        i32.const 20
        local.get 0
        i32.add
        ))
