# Wrausmt

A WASM interpreter written in Rust, as a learning exercise

## Goals

* Avoid using any 3rd party Rust libraries, so that I have to build more things
  on my own.
* Use the official [WASM
  specification](https://webassembly.github.io/spec/core/index.html) to guide
the implementation. 
* Make at least some attempt to make the execution implementation fast, without
  compromising code clarity.
* Organize the code so that it's easy to navigate and explore.

## Current State

At the moment, the implementation is still in the early stages. The binary
parser can parse a fairly simple module containing add, subtract, get/set
local, and const instructions for i32 types.

### Rough TODO/Status list

There will be a single WASM ops file that contains metadata about WASM
operations, and code snippets to execute them given an "execution context". The
plan is to use this file to generate tedious, repetitive code. I'm hoping that this
will help in situations where I might want to refactor instruction
organization, to avoid lots of tediuos code updating.

* Binary parser 
  * Most structure prsent and complete
  * Many instructions not parsed yet.
* Text parser 
  * Lexer complete
  * basic parsing section scaffold in placer.
* Execution engine 
  * Very rough structure present. 
* Instruction implementations for parsing & execution
  * Just a handful of instructions.
* Memory implementation
  * Not started
* Host function implementation
  * Not started
* Module validation
  * Not started
* Spec test execution
  * Not started




