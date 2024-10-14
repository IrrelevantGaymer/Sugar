# Sugar Programming Language

Sugar is a WIP Hobby Programming Language meant for Systems Level Programming.  One of the main features of Sugar is that you can choose between a Garbage Collector or Rust-like Ownership for any particular piece of data.  Sugar is heavily inspired by Rust and the ML family of programming languages such as Ocaml and Haskell.

This README.md file is a temporary solution for Sugar Tutorial and Documentation

## Primitive Data Types

Here is a list of Sugar’s primitive data types: i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize, f32, f64, char, bool, and &str (we’ll took more about this guy later)

i types represent signed integer types, u types represent unsigned integer types, and f types represent floating point types.

The number represents the amount of bits the type uses.  For example, i8 is a signed integer with 8 bits, otherwise known as a byte in C or C++.  isize and usize types’ sizes are dependent on hardware: for 32 bit machines, these size types will be 32 bits, and for 64 bit machines, 64 bits.  So usize is equivalent to size_t in C++.

The &str type is a string slice, where a slice is a fat pointer, a pointer with extra data (in this case a size).  The internal representation of &str is (in C language) { *char, size_t }.  In Layman’s terms, &str represents a string literal.

## Variables

*note: in its current state, variables are declared type first, C-style, however this will change to identifier first, like Rust, Typescript, Go, Jai, Zig, etc.  This is because certain language features will be a lot nicer with identifier first syntax, such as Anonymous Structs as types*

Variables are declared type first similar to C.  For example:

```
i32 count = 0;
```

By default, all variables are immutable.  So every variable is equivalently const or read only.  If you want to mutate a variable, you must add the mut keyword.

```
i32 mut count = 0;
```

Sugar also has type inference.  You can replace the type with the keyword let, making the compiler infer the type for you.

```
let count = 0;
```

## Functions

Every Sugar Program must have an entry point.  The entry point is always the function main that returns nothing.  Like this:

```
fn main {
     ## do stuff
}
```

Functions are defined with the fn keyword.  A name is given, then the arguments, a colon, a return type, then the body:

```
fn add (i32 a, i32 b) : i32 {
     return a + b;
}

fn no_out (i32 x, i32 y) {
     ## do stuff
}

fn no_in : i32 {
     return 3;
}
```

Functions in Sugar have a lot more flexible syntax than most languages.  There are 3 types of functions: prefix, infix, and postfix.  Prefix functions are functions where all their arguments are placed to the right, Infix are when arguments are placed to the left and to the right, and Postfix are when arguments are placed only to the left.  By default, functions are prefix.  To define a function explicitly as prefix, infix, or postfix, the respective keyword must be placed in correspondence to the placement of arguments.  For example, infix is placed in between the arguments on the left and the arguments on the right; prefix is placed before every argument; and postfix is placed after.  So for example, I will create a postfix function that squares an integer:

```
fn square (i32 a) postfix : i32 {
     return a * a;
}
```

To call a function, you don’t use parenthesis around the arguments.  So to call the square function, it would look like this:

```
let a = 3 square;
let b = 2 + 7 square; ## equivalent to 9 square or (2 + 7) square
```

Functions are called left to right, so given two other functions add and foo:

```
fn add (i32 a) infix (i32 b) : i32 {
     return a + b;
}

fn foo (i32 a, i32 b) infix (i32 c) : i32 {
     return a + b * c;
}
```

The following line of code will be equivalent to the next C code

Sugar:
```
1 + (2 3 foo 3 square) square;
```

C
```
square(1, square(foo(2, 3, 3)));
```

Postfix functions are very useful for chaining several functions together.

## Operations

Sugar supports typical operations: +, -, *, /, and %.  However, / only applies to integer types.  For floating point types, // is the correct operation.  ++ also allows for concatenation of group types, such as arrays, lists, strings, and tuples.  Sugar also supports exponentiation **.  There are also typical Bitwise and Logic operators: <<, >>, ~, &, |, ^, !, &&, and ||.  Sugar also has a “logic” xor ^^, which only works on Boolean types, whereas bitwise xor ^ only works on integer and floating point types.  There’s also typical comparison operators: <, >, <=, >=, ==, and !=.

*note: operators will be changed in the future.  All operators operating on floating points will have a . at the end, inspired by Ocaml.  For example 2. +. 3.5, as opposed to 2. + 3.5.  Group type operations will end with +, so ++ is concatenation, *+ is a Cartesian product, etc.  Overloaded Operators end with ` to denote its a custom operator.  For example, an overloaded addition operator on vectors would look like:

```
Vector3 u = Vector3::new 1 1 1;
Vector3 v = Vector3::new -1 2 -1;
Vector3 w = u +` v;
```

I am currently looking into possible alternative symbols instead of `.  I like the look of \*, however that runs into issues with just normal multiplication \* and exponentiation \*\* becoming \*\*, \*\*\* respectively which would be very confusing.*

## Dollar Operator

The Dollar operator is syntactic sugar for wrapping the next expression in parenthesis or other delimiters.  This can make a lot of code look a lot neater.  For example:

```
3 * (2 + x)
```
can be written as
```
3 * $ 2 + x
```

Functions can now be written as (we can also omit commas if we want to:

```
fn add $ i32 a infix $ i32 b : i32 {
     return a + b;
}

fn no_out $ i32 x i32 y {
     ## do stuff
}

fn no_in : i32 {
     return 3;
}

fn foo $ i32 a i32 b infix $ i32 c : i32 {
     return a + b * c;
}

fn square $ i32 a postfix : i32 {
     return a * a;
}
```

(You can also put a dollar before or wrap in parenthesis prefix, infix, or postfix for clarity)

This will also apply to arrays.

```
let mut board = [[0, 1, 0],
                 [0, 2, 1],
                 [2, 2, 0]];
```

Could instead be written as

```
let mut board = $ $ 0 1 0
                  $ 0 2 1
                  $ 2 2 0;
```

## Comments

Because // is an operator, the comment sigil must be something different.  Single line comments are ## and multi line comments start with #, and end with ,#.  For example:

\#\# single line comment
\#,
 , multi line comment line 1
 , line 2
 , line 3
 , \#,
 ,  , nested multi line comments are allowed
 ,  ,\#
 ,\#

*note: once operators are overhauled, comments will be // and /, ,/.  Commas are easier to type than \*.*

## Features to be implemented

Anonymous Structs
Binding, Currying, and Partial Application
Methods
Into operator
Handling multiple files
Standard Build System like cargo
Compiler
Interpreter
Enums
Bitflags
Standard Library
Inline Assembly
Generics and Const Generics
Pattern Matching
FFI interop
Bootstrapping
Standard Library