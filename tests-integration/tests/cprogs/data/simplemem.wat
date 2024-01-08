(module
  (type (;0;) (func))
  (type (;1;) (func (param i32) (result i32)))
  (import "env" "__stack_pointer" (global (;0;) (mut i32)))
  (import "env" "__memory_base" (global (;1;) i32))
  (import "env" "__table_base" (global (;2;) i32))
  (func (;0;) (type 0)
    call 1)
  (func (;1;) (type 0))
  (func (;2;) (type 1) (param i32) (result i32)
    call 3
    i32.const 0
    i32.load offset=1 align=4
    local.get 0
    i32.add)
  (func (;3;) (type 0) 
    i32.const 0
    i32.const 0
    i32.load offset=1 align=4
    i32.const 1
    i32.add
    i32.store offset=1 align=4
  ) 
  (memory (;0;) 100 1000)
  (global (;3;) i32 (i32.const 0))
  (export "__post_instantiate" (func 0))
  (export "test" (func 2))
  (export "inc" (func 3))
  (export "__dso_handle" (global 3))
  (export "__wasm_apply_data_relocs" (func 1)))
