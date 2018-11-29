(module
 (type $0 (func (param i32) (result i32)))
 (type $1 (func (result i32)))
 (type $2 (func))
 (import "env" "memoryBase" (global $gimport$0 i32))
 (import "env" "memory" (memory $0 256))
 (import "env" "table" (table 0 anyfunc))
 (import "env" "tableBase" (global $gimport$4 i32))
 (import "env" "_puts" (func $fimport$1 (param i32) (result i32)))
 (global $global$0 (mut i32) (i32.const 0))
 (global $global$1 (mut i32) (i32.const 0))
 (global $global$2 i32 (i32.const 0))
 (data (i32.const 13) "hello, world!")
 (export "_main" (func $0))
 (start $0)
 (func $0 (type $2)
  (drop
   (call $fimport$1
    (get_global $gimport$0)
   )
  )
 )
)

