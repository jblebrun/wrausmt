(module
  (type (;0;) (func))
  (type (;1;) (func (param i32 i32) (result i32)))
  (func (;2;) (type 1) (param i32 i32) (result i32)
    local.get 1
    local.get 0
    i32.const 1
    i32.eq
    if (result i32)
    i32.const 2
    else
    i32.const 4
    end
    i32.mul) 
  (func (;2;) (type 1) (param i32 i32) (result i32)
    block (result i32)
    i32.const 1
    br 0
    i32.const 2
    end
    i32.const 42
    drop)
  (func (;2;) (type 1) (param i32 i32) (result i32)
    loop (result i32)
    i32.const 1
    end
    i32.const 44
    drop)
  (export "test" (func 0)))
