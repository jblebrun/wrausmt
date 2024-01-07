# Wrausmt

A WASM interpreter written in Rust, as a learning exercise

## Goals

* Avoid using any 3rd party Rust libraries, so that I have to build more things
  on my own. This results in some seemingly silly behavior, like writing my own
  logging and error handling code, rather than using the obvious choices from
  the rust ecosystem. But this is intentional: I want to understand *how* you
  would make things like that, so I make simpler verisons on my own.

* Use the official [WASM
  specification](https://webassembly.github.io/spec/core/index.html) to guide
  the implementation. 

* Make at least some attempt to make the execution implementation fast, without
  compromising code clarity. Since there are zero external dependencies this
  could end up being useful for limited code-size embedding applications. But
  thus far I'm not focusing on reducing code size as a goal.

* Organize the code so that it's easy to navigate and explore, and explore
  different Rust management strategies. It's possible that this code is
  over-crated, but that's intentional.

* Explore interesting rust patterns, new and upcoming nightly features, and try
  different ideas out.


## Current State

At the moment, the implementation is nearly completely. The test and binary
parsers and runtime pass all positive validation tests. The negative validation
tests are a work in progress; as of writing this, all that remains is the
validation tests: trap, malformed, exhaustion tests are all passing.

* [Other notes](./NOTES.md)
