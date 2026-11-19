# The Fudge language specification

Version 0.2.1
(19 Nov 2026)

# Philosophy

# Program Structure

## Program Entry Point

The program entry point for a binary executable is the function with the signature `main() -> U8` or `main(argv: [Str]) -> U8`.

# Variables and Expressions

# Types

## Scalar Types

### Integers


### Floating-point

Floating point types include F64 and F32, these are represented as IEEE754 floats as you would expect from most modern languages.

Floats provide a `:canonicalize_NaN()` method, which converts any valid representation of `NaN` into the same constant value of [still gotta figure that one out]


### Boolean
The boolean type is really just a U8 with only two possible values: `0` and `1`, you can convert from any integer `n` to a boolean
using `n != 0`.

`F32` and `F64` provide a `:is_nonzero()` method, which will compare the float to the float-constants `0.0` or `-0.0`.

## Derived Types
Deried types are any of the types that are not built into the language, like the integer- boolean- and floating point types.
These include all structs, enums and unions.

# Compiler options

## flags
- `--print-mir`, `--print-mir=file`: print the AST after all static analysis is performed, to stdout, stderr or a specified file.
- `--loglevel=LEVEL`: specify the logging level, where `LEVEL` is one of `[ERROR, WARN, INFO, DEBUG, TRACE]` (from least- to most-verbose)

## EXTENSION: Compiler subcommands
The compiler can also setup a project skeleton for you, 

## EXTENSION: Compiler directives
A compiler directive is an annotation above some piece of code that has the form `@ [directive-name] [directive option]`

Some directives are applicable to the compilation of a whole module, while others apply to specific datatypes or functions within a module.

All directives have a default value,
These are the values that are used when compiling using the default flags and when omitting directives in-code.

To change the defaults within a module, you may supply them above the module declaration like so:
```
@ layout-reorder = false
module main
...
```

This will apply the directive to all applicable items within that module.

To change the defaults or directives when compiling multiple modules, directives can be supplied after the `--@` subcommand.
```
fudge compile a.fud b.fud --@ layout-reorder=false target-arch=x86_64-unknown-linux-gnu
```
This will compile both `a.fud` and `b.fud` as if their declaration had `@ layout-reordering = false` above them.

alternatively, a path to a `.toml` file containing directives can be supplied instead:
```
// flags.toml
layout-reordering = false
target-arch = "x86_64-unknown-linux-gnu"
...
// flags.toml

> fudge compile a.zea b.zea --@ file=flags.toml
```


The compiler will search for a `flags.toml` file within the same directory that the compiler is called from,
if it finds such a file, it will apply those flags to the compilation.

These directives instruct the compiler to do something at compile-time, such as:
- `@ struct-reorder = true|false` for specifying struct layout ordering
- `@ target-triple = [target triple]` for specifying the target architecture
- `@ alignment = N` for specifying the alignment of a datatype in bytes
- `@ inline` for inlining
- `@ hot` and/or `@ cold` for hinting hot- or cold branches
- `@ unreachable` for specifying unreachable code branches code that must otherwise be exhaustive

Supplying an non-existent directive, or supplying a directives with an invalid value such as `@ alignment = shimmadingle` will result in a compilation failure.

## EXTENSION: profiles
A `profiles.toml` file containing profiles can be supplied, along with a profile upon compilation.
```
// profiles.toml

default-profile = debug

[optimize]
Olevel = 3
strip-debug = true
debug-assert = "ignore"
abort-unreachable = false

[debug]
Olevel = 0
strip-debug = false
debug-assert = true
```

a profile can be chosen at compile-time using the `--profile=[profile]` flag, if it is omitted, the `default-profile` form the profiles is used.

## Typing Rules

| binop                              | typeof(lhs) | typeof(rhs) | inferred type                             |
|------------------------------------|-------------|-------------|-------------------------------------------|
| `+`,`-`,`*`,`/`,`%`, `&`, `\|`,`^` | `Int(s1)`   | `Int(s2)`   | `Int(max(s1,s2))` (zero extension rule)   |
| `+`,`-`,`*`,`/`,`%`                | `Float(s1)` | `Float(s2)` | `Float(max(s1,s2))` (zero extension rule) |
| `&&`,`\|\|`,`^^`                   | `Bool`      | `Bool`      | `Bool`                                    |
| `[]` (indexing)                    | `[t]`       | `Int(s)`    | `t`                                       |
| `.` (member access)                | `t`         | ...         | `t::m` (struct lookup)                    |

| unop | typeof(arg) | inferred type |
|------|-------------|---------------|
| `-`  | `Int(s)`    | `Int(s)`      |
| `-`  | `Float(s)`  | `Float(s)`    |
| `~`  | `Int(s)`    | `Int(s)`      |
| `~`  | `Bool`      | `Bool`        |

## type conversion rules/conditions
- An integer type can always be cast into a wider integer type of the same sign: `u8 -> u16|u32|u64` and `i8 -> i16|i32|i64`.
- An unsigned integer type can always be case to a wider signed type: `u8 -> i16|i32|i64`
- A `signed -> unsigned` cast must always be explicit, and is identical to a bit-reinterpretation (i.e. `-1 -> 255`)
- A boolean may always be cast into any integer type, where `true -> 1` and `false -> 0`
- An `integer -> boolean` cast must be explicit, and is equal to checking for non-equality with `0`, i.e. `int != 0`
- An integer cast to a more narrow integer type must be explicit, and is identical to truncation.
- An integer cast of the form `Uw1 -> Iw2` or `Iw1 -> Uw2` where `w1 < w2` (a widening cast that also changes the sign)
  will first perform the sign-cast, then the widening cast: `I8 -> U16 === I8 -> U8 -> U16`
- An `F32 -> F64` cast is always allowed, the other way around must be explicit however.
- A `boolean -> float` cast must be explicit, where `true -> 1.0f64` and `false -> 0.0f64`
- Pointer to different types may never be cast into one another: `*U8 -> *I8` will always throw an error,
  use a union if you wish to achieve behaviour like this

The standard library provides a module called bit-reinterpret,
which will be populated at compile-time with function that bit-cast between types. Cast functions will only be synthesized for types of equal width.

## Struct size and field ordering
Struct will have an alighment equal to that of its field with the highest alignment.
The size `S` of a struct is calculated as follows:
- iterate over each field `f` with alignment `a`
- align the current size to a multiple of `a`, then add the size of `f`
When `@ struct-reorder = true`, the fields of a struct are placed in descending order of size.

## C's undefined behaviour compared to Fudge
- unreachable code
    - reaching code marked `@ unreachable` when `@ abort-unreachable = true` (the default) will immediatly crash the program with an error,
    - reaching code marked `@ unreachable` when `@ abort-unreachable = false` will depend entirely on the generated machine code and is considered undefined,
      it may lead to memory corruptions or explosions or something, be careful.
- bit-casting between types of different sizes using `bit-reinterpret<val, Type>` depends:
    - casting to a wider type will zero-extend the value with most-significant bytes
    - casting to a narrower type will truncate the excess bits
- dereferencing a NULL pointer will immediatly crash the program with an error
- initialization
    - allocated memory is always initialized to 0 when the `@ zero-initialize = true` (the default) is set
    - allocated memory is left unitialized when `@ zero-initialize = false`



# Grammar

```bnf
<module> = 
    "module" <expr ident>
    ( "imports" "{" <expr ident>,+  "}" )?
    ( "exports" "{" <expr ident>,+  "}" )?
    <module items>
    
<module items> =
    <function definition> <module items>
|   <initialisation> <module items>

<initialisation> =
     <assignment pattern> ":" <type specifier>? "=" <expression> ";"

<reassignment> =
     <expr ident> "=" <expression> ";"


<assignment pattern> =
    <expr ident> // simple assignee
|   "(" <expr ident>,+ ")" // unpacking assignee

<type specifer> =
    <type ident> // basic type
|   <type ident> "*" // pointer type
|   "[" <type ident> "]" // array type

<function definition> =
    "func" <expr ident> 
    "(" <typed ident>,* ")"
    "->" <type specifier> 
    <statement block>

<typed ident> =
    <expr ident> ":" <type specifier>

<struct definition> = 
    "struct" <type ident> "{"
        <typed identifier>,+
    "}"

<if branch> =
    "if" <expression> <statement block>
|   "if" <expression> <statement block> "else" <statement block>


<expression> = 
#precedence 0
    <expression> "||" <expression>
#precedence 1
    <expression> "^^" <expression>
#precedence 2
    <expression> "&&" <expression>
#precedence 3
    <expression> "|" <expression>
#precedence 4
    <expression> "^" <expression>
#precedence 5
    <expression> "&" <expression>
#precedence 6
    <expression> "==" <expression>
|   <expression> "!=" <expression>
#precedence 7
    <expression> "<" <expression>
|   <expression> ">" <expression>
|   <expression> "<=" <expression>
|   <expression> ">=" <expression>
#precedence 8 (left associative)
    <expression> "<<" <expression>
|   <expression> ">>" <expression>
#precedence 9 (left associative)
    <expression> "+" <expression>
|   <expression> "-" <expression>
#precedence 10 (left associative)
    <expression> "*" <expression>
|   <expression> "/" <expression>
|   <expression> "%" <expression>
#precedence 11
    "!" <expression>
|   "-" <expression>
|   "~" <expression>
#precedence 12
    <expression> "." <expr ident>
|   <expression> "[" <expression> "]"
|   <function call>
#precedence 13
    <expr ident>
|   <integer literal>
|   <float literal>
|   "true" | "false"

<statement> =
    <initialisation>
|   <function call> ";"
|   "return" <expression> ";"
|   <reassignment>
|   <statement block>

<function call> =
    <expr ident> "(" <expression>,* ")"

<statement block> =
    "{" <statement>+  <expression>? "}"

<numeric literal> =
    regex/(0d)?[_0123456789]+/i
|   regex/0x[_0123456789abcdef]+/i
|   regex/0b[01][_01]*/i

<float literal> =
#helper <sign> = "+" | "-"
#helper  <e> = "e" | "E"
#helper <exponent> = <e> <sign>? regex/[0123456789]+/
    <sign>? regex/[0123456789]+/ "." <exponent>?
|   <sign>? "." regex/[0123456789]+/ <exponent>?
|   <sign>? regex/[0123456789]+/ "." regex/[0123456789]+/ <exponent>?
    
```
