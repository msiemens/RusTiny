# RusTiny

This is an educational compiler for a Rust-like language that targets the
fictional [Tiny architecture](https://github.com/msiemens/rust-tinyasm).
The syntax is based on Rust, but there are numerous semantic differences:

- The only datatype is `int`. There also are `bool` and `chair`, but these are
  actually `int`s in disguise.
- No structs/classes, no modules, only functions. This keeps the whole
  language managable for me.

## Goal

My goal is to get the compiler so far that I can write a program that
[approximates Pi](blog.m-siemens.de/exploring-computers-tiny-assembler/#approximatingdpid).

## Architecture

The general data flow looks something like this:

    Source File -(front)-> AST -(middle)-> IR -(back)-> Assembler

- `front`: Translates the source file into an Abstract Syntax Tree representation
- `middle`: Checks the AST for correctness, transforms it to an Intermediate Representation
  and performs optimizations
- `back`: Translates the IR to Tiny Assembly code

## Helpful Resources

Resources I found helpful:

- [**Introduction to Compiler Design** by Torben Ægidius Mogensen](http://www.springer.com/us/book/9780857298287)
- [**Engineering a Compiler** by Keith D. Cooper & Linda Torczon](http://store.elsevier.com/product.jsp?isbn=9780120884780)
- [**Single-Pass Generation of Static Single Assignment Form for Structured Languages** by MARC M. BRANDIS and HANSPETER MÖSSENBÖCK](http://www-plan.cs.colorado.edu/diwan/7135/p1684-brandis.pdf)