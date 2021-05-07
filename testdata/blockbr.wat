(module
  (func $simpleblock (result i32) 
    (block (result i32)
        i32.const 14
        br 0
        drop
        i32.const 12
    ))
  (func $simpleblockplain (result i32) 
    block (result i32)
        i32.const 114
        br 0
        drop
        i32.const 112
    end)

  (func $nestedblock (result i32) 
    (block (result i32)
      (block (result i32)
        i32.const 14
        br 0
        drop
        i32.const 12
      )
      drop
      i32.const 13
    ))
 (func $nestedblockplain (result i32) 
    block (result i32)
      block (result i32)
        i32.const 114
        br 0
        drop
        i32.const 112
      end
      drop
      i32.const 113
    end)

    (func (export "nestedbreakcleanup") (result i32) 
          (block (result i32)
                 i32.const 14
                 (block (result i32)
                    i32.const 10
                    i32.const 20
                    br 0
                 )
                 i32.add
        ))
    (func (export "nestedbreakcleanupparams") (result i32) 
          (block (result i32)
                 i32.const 14
                 i32.const 100
                 (block (param i32) (result i32)
                    i32.const 10
                    i32.const 20
                    br 0
                 )
                 i32.add
        ))
  (export "simpleblock" (func $simpleblock))
  (export "nestedblock" (func $nestedblock))
  (export "simpleblockplain" (func $simpleblockplain))
  (export "nestedblockplain" (func $nestedblockplain)))

             
