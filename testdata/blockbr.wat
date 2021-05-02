(module
  (func $simpleblock (result i32) 
    (block
        i32.const 14
        br 0
        drop
        i32.const 12
    ))
  (func $simpleblockplain (result i32) 
    block
        i32.const 114
        br 0
        drop
        i32.const 112
    end)

  (func $nestedblock (result i32) 
    (block
      (block
        i32.const 14
        br 0
        drop
        i32.const 12
      )
      drop
      i32.const 13
    ))
 (func $nestedblockplain (result i32) 
    block
      block
        i32.const 114
        br 0
        drop
        i32.const 112
      end
      drop
      i32.const 113
    end)
  (export "simpleblock" (func $simpleblock))
  (export "nestedblock" (func $nestedblock))
  (export "simpleblockplain" (func $simpleblockplain))
  (export "nestedblockplain" (func $nestedblockplain)))

             
