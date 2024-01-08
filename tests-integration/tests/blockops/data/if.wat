(module
  (func $simpleif (param i32) (result i32) 
    local.get 0
    if (result i32)
        i32.const 0x42
    else 
        i32.const 0x43
    end)
  (func $nestedblockif (param i32) (param i32) (result i32) 
    (if (result i32) (local.get 0)
      (then (if (result i32) (local.get 1)
              (then i32.const 0x42)
              (else i32.const 0x43)))
      (else (if (result i32) (local.get 1)
              (then i32.const 0x142)
              (else i32.const 0x143)))))
  (func $nestedif (param i32) (param i32) (result i32) 
    local.get 1
    local.get 0
    if (param i32) (result i32) 
        if (result i32) 
            i32.const 0x42
        else 
            i32.const 0x43
        end
    else 
        if (result i32) 
            i32.const 0x142
        else
            i32.const 0x143
        end
    end)
  (export "nestedif" (func $nestedif))
  (export "nestedblockif" (func $nestedblockif))
  (export "simpleif" (func $simpleif)))

             
