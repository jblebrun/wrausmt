(module
  (func (export "get32") (param i32) (result i32)
    local.get 0
    i32.load offset=0
  ) 
  (func (export "get64") (param i32) (result i64)
    i32.const 0
    i64.load offset=0
  ) 
  (func (export "get32_f") (param i32) (result f32)
    local.get 0
    f32.load offset=0
  ) 
  (func (export "get64_f") (param i32) (result f64)
    local.get 0
    f64.load offset=0
  ) 
  (func (export "get32_8u") (param i32) (result i32)
    local.get 0
    i32.load8_u offset=0
  ) 
  (func (export "get32_8s") (param i32) (result i32)
    local.get 0
    i32.load8_s offset=0
  ) 
  (func (export "get32_16u") (param i32) (result i32)
    local.get 0
    i32.load16_u offset=0
  ) 
  (func (export "get32_16s") (param i32) (result i32)
    local.get 0
    i32.load16_s offset=0
  ) 
  (func (export "get64_8u") (param i32) (result i64)
    local.get 0
    i64.load8_u offset=0
  ) 
  (func (export "get64_8s") (param i32) (result i64)
    local.get 0
    i64.load8_s offset=0
  ) 
  (func (export "get64_16u") (param i32) (result i64)
    local.get 0
    i64.load16_u offset=0
  ) 
  (func (export "get64_16s") (param i32) (result i64)
    local.get 0
    i64.load16_s offset=0
  ) 
  (func (export "get64_32u") (param i32) (result i64)
    local.get 0
    i64.load32_u offset=0
  ) 
  (func (export "get64_32s") (param i32) (result i64)
    local.get 0
    i64.load32_s offset=0
  ) 
  (func (export "put64_8") (param i32) (param i64)
    local.get 0
    local.get 1
    i64.store8 offset=0
  ) 
  (func (export "put64_16") (param i32) (param i64)
    local.get 0
    local.get 1
    i64.store16 offset=0
  ) 
  (func (export "put64_32") (param i32) (param i64)
    local.get 0
    local.get 1
    i64.store32 offset=0
  ) 
  (func (export "put32_f") (param i32) (param f32)
    local.get 0
    local.get 1
    f32.store offset=0 
  ) 
  (func (export "put64_f") (param i32) (param f64)
    local.get 0
    local.get 1
    f64.store offset=0 
  ) 
  (func (export "put32") (param i32) (param i32)
    local.get 0
    local.get 1
    i32.store offset=0 
  ) 
  (func (export "put64") (param i32) (param i64)
    local.get 0
    local.get 1
    i64.store offset=0 
  ) 
  (func (export "put32_8") (param i32) (param i32)
    local.get 0
    local.get 1
    i32.store8 offset=0
  ) 
  (func (export "put32_16") (param i32) (param i32)
    local.get 0
    local.get 1
    i32.store16 offset=0
  ) 
  (memory 100 1000)
)
