# Sugar Programming Language

Sugar is a WIP Hobby Programming Language meant for Systems Level Programming.  One of the main features of Sugar is that you can choose between a Garbage Collector or Rust-like Ownership for any particular piece of data.  Sugar is heavily inspired by Rust and the ML family of programming languages such as Ocaml and Haskell.

This README.md file is a temporary solution for Sugar Tutorial and Documentation

## Primitive Data Types

Here is a list of Sugar’s primitive data types: i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize, f32, f64, char, bool, and &str (we’ll talk more about this guy later)

i types represent signed integer types, u types represent unsigned integer types, and f types represent floating point types.

The number represents the amount of bits the type uses.  For example, i8 is a signed integer with 8 bits, otherwise known as a byte in C or C++.  isize and usize types’ sizes are dependent on hardware: for 32 bit machines, these size types will be 32 bits, and for 64 bit machines, 64 bits.  So usize is equivalent to size_t in C++.

The &str type is a string slice, where a slice is a fat pointer, a pointer with extra data (in this case a size).  The internal representation of &str is (in C language) { *char, size_t }.  In Layman’s terms, &str represents a string literal.

## Variables

Variables are declared with the let keyword, followed by it's name.  For example:

```
let count = 0;
```

By default, all variables are immutable.  So every variable is equivalently const or read only.  If you want to mutate a variable, you must add the mut keyword.

```
let mut count = 0;
```

Sugar also has type inference.  You can explictly declare the type of the variable by adding a ':' after the name followed by the desired type.

```
let count: i32 = 0;
```

## Operations

Sugar supports typical operations: +, -, *, /, and %.  However, the operators just listed only apply to integer types.  For floating point types, you append '.' to the operator (i.e. instead of 2.0 + 3.0, it would be 2.0 +. 3.0).  Sugar also supports exponentiation ** (exponentiation for floating points is **.).  There are also typical Bitwise and Logic operators: <<, >>, ~, &, |, ^, !, &&, and ||.  There’s also typical comparison operators: <, >, <=, >=, ==, and !=.  Comparison operators do not have a floating point variant.

*note: Group type operations will end with +, so ++ is concatenation, *+ is a Cartesian product, etc.  Overloaded Operators end with ` to denote its a custom operator.  For example, an overloaded addition operator on vectors would look like:

```
Vector3 u = Vector3::new 1 1 1;
Vector3 v = Vector3::new -1 2 -1;
Vector3 w = u +` v;
```

I am currently looking into possible alternative symbols instead of `.  I like the look of \*, however that runs into issues with just normal multiplication \* and exponentiation \*\* becoming \*\*, \*\*\* respectively which would be very confusing.*

# Conditionals and While Loops

You can't have a language without branching and looping right?  We have if/else if/else statements and while statements.

Unlike C, you do not need parenthesis around the condition, so this is perfectly acceptable (and preffered):

```
if x < 69 {
     print_string ":(\n";
}
```

You can, like normal, chain else if and else branches.

```
if x == 0 {
     print_string "dead";
} else if x == 1 {
     print_string "a loser";
} else if x == 2 {
     print_string "a couple";
} else {
     print_string "otherwise";
}
```

You can also make a while loop, again foregoing the parenthesis

```
let mut sum = 0;
print_string "input a positive number: ";
let mut input = read_i32;
while input.value > 0 {
     sum += input.value;

     print_string "input a positive number: ";
     input = read_i32;
}
print_string "total: ";
print_i32 sum;
```

## Functions

Every Sugar Program must have an entry point.  The entry point is always the function main that returns nothing.  Like this:

```
fn main {
     ## do stuff
}
```

Functions are defined with the fn keyword.  A name is given, then the arguments, a equals sign '=', a return type, then the body:

```
fn add (a: i32, b: i32) = i32 {
     return a + b;
}

fn no_out (x: i32, y: i32) {
     ## do stuff
}

fn no_in : i32 {
     return 3;
}
```

Functions in Sugar have a lot more flexible syntax than most languages.  There are 3 types of functions: prefix, infix, and postfix.  Prefix functions are functions where all their arguments are placed to the right, Infix are when arguments are placed to the left and to the right, and Postfix are when arguments are placed only to the left.  By default, functions are prefix.  To define a function explicitly as prefix, infix, or postfix, the respective keyword must be placed in correspondence to the placement of arguments.  For example, infix is placed in between the arguments on the left and the arguments on the right; prefix is placed before every argument; and postfix is placed after.  So for example, I will create a postfix function that squares an integer:

```
fn square (a: i32) postfix = i32 {
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
fn add (a: i32) infix (b: i32) = i32 {
     return a + b;
}

fn foo (a: i32, b: i32) infix (c: i32) = i32 {
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

## Structs

You can define structs using the keyword struct, the name, and in braces the structs fields.  Each field must have an accessor such as pub, prv, or pkg.  Fields are declared with the name of the field, a colon ':', then the type.  Each field must be separated by a comma (trailing commas are allowed).

```
struct Coord {
     pub x: i32,
     pub y: i32
}
```

Structs don't have a constructor.  You can only initialize structs by defining all of its fields at once, by listing each field followed by a colon and an expression attached to each one. 
 You can forego the expression if you already have a variable with the field's name.  You can wrap this process in a function called new.

```
pub fn zero_coord = Coord {
     return Coord { x: 0, y: 0 };
}

pub fn new_coord $ x: i32, y: i32 = Coord {
     return Coord { x, y };
}

pub fn add_coords $ a: Coord $ infix $ b: Coord = Coord {
     let x = a.x + b.x;
     let y = a.y + b.y;
     return Coord { x, y };
}
```

You can access a struct's field like normal C-style, with a dot and the field's name

```
let horizontal = coord.x;
```

## Dollar Operator

The Dollar operator is syntactic sugar for wrapping the next expression in parenthesis or other delimiters.  This can make a lot of code look a lot neater.  For example:

```
3 * (2 + x)
```
can be written as
```
3 * $ 2 + x
```

Functions can now be written as:

```
fn add $ a: i32 infix $ b: i32 = i32 {
     return a + b;
}

fn no_out $ x: i32, y: i32 {
     ## do stuff
}

fn no_in = i32 {
     return 3;
}

fn foo $ a: i32, b: i32 b infix $ c: i32 = i32 {
     return a + b * c;
}

fn square $ a: i32 postfix = i32 {
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

## Anonymous Structs

Sometimes we want to be able to group data together, but we only need this specific type of grouping for one thing.  Instead of defining a whole new struct, we can create a temporary one.  When defining the type of a variable or argument/return type of a function, you can list out fields and types of this temporary struct.  These temporary structs are called anonymous structs.  Here are some examples:

```
pub fn div $ a: i32 $ infix $ b: i32 = { value: i32, success: bool } {
     if b == 0 {
          return { value: 0, success: false };
     }
     return { value: a / b, success: false };
}

pub fn main {
     let div: { value: i32, success: bool } = 3 div 0;
     if parse.success {
          print_i32 parse.value;
     } else {
          print_string "divide by zero error";
     }
}
```

You initialize them like you would normal structs, except the anonymous struct doesn't have a name.  You can also access fields of the anonymous struct like normal.

## Comments

For single line comments, Sugar uses typical C-style comments with two forward slashes "//".  However, Multiline comments, instead of being "/\*" and "\*/", are "/," and ",/" simply because I think they are easier to type.

// single line comment\
/,\
 , multi line comment line 1\
 , line 2\
 , line 3\
 , /,\
 ,  , nested multi line comments are allowed\
 ,  ,/\
 ,/

## Built In Functions

*note: these are temporary.  In the roadmap of this language, eventually Sugar will have a standard library and will send its own sys calls to the operating system for IO functions and other things like that.  These Built In Functions are a temporary stand in so that users can try out the language and build little toy programs.

There are a few built in functions into sugar for IO operations: print_string, print_i32, read_char, read_i32, and panic.

print_string will print string literals without a newline, so make sure to add '\n' at the end of your print_string statements.
print_i32 is self-explanatory.
read_char atm can only read ascii characters.  Eventually, there will be support for UTF-8 characters.
read_i32 will read in an integer.
panic will throw an error and abort the program.  Sugar has no intention of featuring exceptions or unwinding errors in any context.  The predominant way to handle errors will be by value.  In the meantime, in lieu of generics and sum types, anonymous structs with a value and a success field will be the way to do this until more features are implemented.

## Roadmap

Completely finish better error messaging\
Add Traits, Impls, and Methods -- note: we do this first so we can define Copy, MoveOnly, Linear and Pin types with the trait system\
Add Generics\
Create built in Garbage Collector and other Allocators and Move semantics (Not everything will be copy!!! yay!!!)\
Add support for Refs and Lifetimes\
Aliasing (actual descriptive names with shorthands so your code isn't an essay)\
Add support for unsafe\
Create built in functions for pointer read/write operations\
Add Inline Assembly\
Multiple Files and Accessors -- at some point during all that mess above\
Directives to set defaults for less annoyance (such as default accessor annotations, lifetime, oxy, etc.)\
Then maybe the compiler teehee\
\
Binding, Currying, and Partial Application\
Into operator\
Linear Types\
Standard Build System like cargo\
Enums\
Bitflags\
Standard Library\
Pattern Matching\
FFI interop\
Bootstrapping\
Standard Library\
