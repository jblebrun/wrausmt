(module
  (type (;0;) (func))
  (type (;1;) (func (param i32) (result i32)))
  (func (;0;) (type 0)
    call 1)
  (func (;1;) (type 0))
  (func (;2;) (type 1) (param i32) (result i32)
    local.get 0
    call 3 
    local.get 0
    i32.add)
  (func (;3;) (type 1) (param i32) (result i32)
    global.get 1
    local.get 0
    i32.add)
    
  (global (;0;) i32 (i32.const 0))
  (global (;1;) i32 (i32.const 0x77))
  (export "__post_instantiate" (func 0))
  (export "test" (func 2))
  (export "__wasm_apply_data_relocs" (func 1)))
