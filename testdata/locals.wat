(module
  (type $void (func))
  (type (;1;) (func (param i32) (result i32)))
  (type (;2;) (func (param $x i32) ))
  (type (;3;) (func (param funcref) ))
  (type (;4;) (func (param externref) ))
  (import "env" "__stack_pointer" (global (;0;) (mut i32)))
  (import "env" "__memory_base" (global (;1;) i32))
  (import "env" "__table_base" (global (;2;) i32))
  (import "env" "memory" (memory (;0;) 0))
  (func $init (type 2) (param i32) 
    call 1)
  (func (type $void))
  (func $foo (param $foo i32) (result i32)
    (local i32)
    i32.const 600
    local.set 1
    local.get 1
    (i32.add (i32.const 2) (i32.const 4))
    i32.sub
    local.get 0
    i32.add
    return)
  (func $inline-export (export "fooexport") (result i32)
    i32.const 45)
  (func
    br_table 0 4 5 2 local.get 0)
  (func
    call_indirect 0 (param i32) (result i32))
  ;; Inline type
  (global (;3;) i32 (i32.const 0))
  (export "__post_instantiate" (func 0))
  (export "test" (func $foo))
  (export "__dso_handle" (global 3))
  (export "__wasm_apply_data_relocs" (func 1)))
