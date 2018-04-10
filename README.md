# RusTiny
[![FOSSA Status](https://app.fossa.io/api/projects/git%2Bgithub.com%2Fmsiemens%2FRusTiny.svg?type=shield)](https://app.fossa.io/projects/git%2Bgithub.com%2Fmsiemens%2FRusTiny?ref=badge_shield)


This is an educational compiler for a Rust-like language that was originally
planned to target the fictional [Tiny architecture](https://github.com/msiemens/rust-tinyasm). But due to Tiny's limitations, RusTiny now targets x86-64 machines.
The syntax is based on Rust, but there are numerous semantic differences:

- The only datatype is `int`. There also are `bool` and `char`, but these are
  actually `int`s in disguise.
- No structs/classes, no modules, only functions. This keeps the whole
  language managable for me.
- No `mut`, no borrow checker. Again: keep it simple.

**NOTE:** This project is currently on hold as the current approach for SSA register allocation doesn't really pan out. I hope I'll find time to revisit project this sometime in the future.

## Goal

My goal is to get the compiler so far that I can write a program that
[approximates Pi](https://blog.m-siemens.de/exploring-computers-tiny-assembler/#approximatingdpid).

## Architecture

The general data flow looks something like this:

    Source File -(front)-> AST -(middle)-> IR -(back)-> Assembler

- `front`: Translates the source file into an Abstract Syntax Tree representation
- `middle`: Checks the AST for correctness, transforms it to an Intermediate Representation
  and performs optimizations
- `back`: Translates the IR to ~~Tiny Assembly code~~ x86-64 assembly

## Helpful Resources

Resources I found helpful:

- [**Introduction to Compiler Design** by Torben Ægidius Mogensen](http://www.springer.com/us/book/9780857298287)
- [**Engineering a Compiler** by Keith D. Cooper & Linda Torczon](http://store.elsevier.com/product.jsp?isbn=9780120884780)


## License
[![FOSSA Status](https://app.fossa.io/api/projects/git%2Bgithub.com%2Fmsiemens%2FRusTiny.svg?type=large)](https://app.fossa.io/projects/git%2Bgithub.com%2Fmsiemens%2FRusTiny?ref=badge_large)