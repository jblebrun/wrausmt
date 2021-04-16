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

* [TODO](./TODO.md)
* [Other notes](./NOTES.md)
