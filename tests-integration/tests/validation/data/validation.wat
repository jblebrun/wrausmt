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

(module (func (result i32) (block (result i32) (i32.const 42))))

(module 
 (func (export "simple-if") (param i32) (result i32)
    (if (result i32) (local.get 0) (then (i32.const 7)) (else (i32.const 8))))
)
(assert_return (invoke "simple-if" (i32.const 0)) (i32.const 8))
(assert_return (invoke "simple-if" (i32.const 1)) (i32.const 7))

(module (func (export "type-i32-value") (result i32)
    (block (result i32) (i32.ctz (br 0 (i32.const 1))))
))
(assert_return (invoke "type-i32-value") (i32.const 1))

(module (func (export "as-loop-first") (result i32)
    (block (result i32) (loop (result i32) (br 1 (i32.const 3)) (i32.const 2)))
))

(assert_return (invoke "as-loop-first") (i32.const 3))

(module (func (export "type-i32-value") (result i32)
    (block (result i32) (i32.ctz (br_if 0 (i32.const 1) (i32.const 1))))
))

  (func (export "singleton") (param i32) (result i32)
    (block
      (block
        (br_table 1 0 (local.get 0))
        (return (i32.const 21))
      )
      (return (i32.const 20))
    )
    (i32.const 22)
  )

  (module
  (memory (export "memory0") 1 1)
  (data (i32.const 2) "\03\01\04\01")
  (data "\02\07\01\08")
  (data (i32.const 12) "\07\05\02\03\06")
  (data "\05\09\02\07\06")
  (func (export "test")
    (memory.init 1 (i32.const 7) (i32.const 0) (i32.const 4))
    (data.drop 1)
    (memory.init 3 (i32.const 15) (i32.const 1) (i32.const 3))
    (data.drop 3)
    (memory.copy (i32.const 20) (i32.const 15) (i32.const 5))
    (memory.copy (i32.const 21) (i32.const 29) (i32.const 1))
    (memory.copy (i32.const 24) (i32.const 10) (i32.const 1))
    (memory.copy (i32.const 13) (i32.const 11) (i32.const 4))
    (memory.copy (i32.const 19) (i32.const 20) (i32.const 5))
    (memory.fill (i32.const 19) (i32.const 20) (i32.const 5)))
  (func (export "load8_u") (param i32) (result i32)
    (i32.load8_u (local.get 0))))


(module
  (memory 1)
  (func (export "grow") (param i32) (result i32)
    (memory.grow (local.get 0))
  ))

(module
  (memory 1)
  (func (export "size") (result i32)
    (memory.size)
  ))

(module
  (func (export "ef0") (result i32) (i32.const 0))
  (func (export "ef1") (result i32) (i32.const 1))
  (func (export "ef2") (result i32) (i32.const 2))
  (func (export "ef3") (result i32) (i32.const 3))
  (func (export "ef4") (result i32) (i32.const 4))
)

(register "a")
(module
  (import "a" "ef0" (func (result i32)))    ;; index 0
  (import "a" "ef1" (func (result i32)))
  (import "a" "ef2" (func (result i32)))
  (import "a" "ef3" (func (result i32)))
  (table $t0 30 30 funcref)
  (table $t1 30 30 funcref)
  (elem (table $t0) (i32.const 2) func 0 1 2 1)
  (func (result i32) (i32.const 4))    ;; index 4
  (func (result i32) (i32.const 5))
  (func (result i32) (i32.const 6))
  (func (result i32) (i32.const 7))
  (elem funcref
    (ref.func 3) (ref.func 5) (ref.func 2) (ref.func 4) (ref.func 1))
  (elem funcref
    (ref.func 5) (ref.func 5) (ref.func 2) (ref.func 4) (ref.func 1))
  (func (export "test")
    (elem.drop 2)
    (table.init $t0 1 (i32.const 2) (i32.const 0) (i32.const 4)))
)

(module
  (table $t 0 externref)
  (table $t2 0 externref)

  (func (export "grow") (param $sz i32) (param $init externref) (result i32)
    (table.grow $t (local.get $init) (local.get $sz))
  )
  (func (export "fill") (param $i i32) (param $n i32)
    (table.fill $t2 (local.get $i) (ref.null extern) (local.get $n)))

  (func (export "grow-abbrev") (param $sz i32) (param $init externref) (result i32)
    (table.grow (local.get $init) (local.get $sz))
  )

  (func (export "size") (result i32) (table.size $t))

  (func (export "copy") (table.copy $t $t2 (i32.const 0) (i32.const 1) (i32.const 5)))
)

(module
  (func $f1 (export "funcref") (param $x funcref) (result i32)
    (ref.is_null (local.get $x))
  )
  (func $f2 (export "externref") (param $x externref) (result i32)
    (ref.is_null (local.get $x))
  )
)

(module
  (func (result i32) (select (i32.const 1) (i32.const 2) (i32.const 0)))
  (func (result i32) (select (result i32) (i32.const 1) (i32.const 2) (i32.const 0)))
)

(module 
 (type $proc (func))
 (type $out-i32 (func (result i32)))
 (table funcref (elem $f-i32))
 (func $f-i32 (result i32) (i32.const 42))
 (func (export "type-i32") (result i32)
    (nop)
    (call_indirect (type $out-i32) (i32.const 0))
  )
)

(module 
   (func $const-i32 (result i32) (i32.const 0x132))
   (func (export "type-i32") (result i32) (call $const-i32))
)