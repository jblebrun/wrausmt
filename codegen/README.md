WebAssembly instruction handler code generation

master_ops_list.csv contains a list of each webassembly opcode, and the code
snippet needed to execute it given a set of `ExecutionContextActions`.

The goal of this setup is to make it easy to do structural refactors later, if
desired.
