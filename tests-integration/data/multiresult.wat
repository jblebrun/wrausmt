(module
  (func (result i32 i64)
    i32.const 0x42
    i64.const 0xFF42
  )

  (func (export "test") (result i64 i32) (local i32 i64)
    call 0
    local.set 0
    local.set 1
    local.get 0
    local.get 1
    )
)
