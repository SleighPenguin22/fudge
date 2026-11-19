# Fudge

Fudge is a systems language that aims to provide developer ergonomics, while keeping close to the performance
of established languages like C or Rust.

The syntax looks quite similar to Rust, mostly because I like Rust's syntax :D

# working features

- emitting global integers into the object files
- returning integer -literals, -locals and -globals from `main()`
- thats about it right now

# how to build/use
- clone the project
- run `cargo build --release`
- run `./target/release/fudge-driver --help` to show CLI usage

## Features borrowed from modern languages
Some features include:
### tuples and destructuring
`fn make-coord(x: F32, y: F32) -> (F32,F32) { (x,y) }`

### tuple-destructuring:
to destructure a tuple, a tuple-pattern is used:
`@(x,y) : (F32,F32) = coord;`
`@(a,b,(c,d)) := some-tuple;`

currently the `@` before a destructuring pattern is requires due to a limitation
in the implementation of the parser. When hand-writing the parser, I hope to resolve this.


### order of declaration is not significant
no more forward declarations!

```Rust
  fn foo() -> U32 {bar()}
  
  fn bar() -> U32 {3}
// this is allowed just fine!
```

### expression-oriented language
```Rust
fn square(x: U32) -> U32 {
    x * x // tailing expression in a block is treated as return
}

fn read-to-string(path: String) -> String* {
    // option type and early return
    f : File = fs:open(path) else return nil; 
    buffer := String:new();
    fs:drain-into!(buffer, f);
    buffer
}

fn foo() -> U32 {
    a := {
        b:= 3; 
        b
    };
    a + 1;    
}
```

## defining datatypes
Fudge allows three kinds of datatypes: structs, tags and unions,
with plans to add syntax-level support for tagged-unions (i.e. Rust enums) later.

structs are a heterogenously-typed collection of named fields.

tags are like C-enums, literally just integers with a name, except that they are in a namespace now!

Unions allow you to read and write to a single variable as if it were many different types.
They are identical to C-unions, except that type-punning (explained later) is well-defined now.

#### Structs
to define a struct, use the `struct` keyword, define fields by declaring variables without the `= value` part:
```
struct Foo {
    field1: U32,
    field2: Bool,
    field3: String*, // trailing comma's are allowed of course
}
```

Instantiating a struct looks similar to defining one, except that you *do* include the `= value` part,
it is not necessary to specify the type.

```
s := "meow";

foo := Foo {
    field1 := 3,
    field2 := true,
    field3 := s&
};
```

#### Tags
To define a tag, use the `tag` keyword, by default, the tags are assigned integer values in increasing order, starting from 0.
You may assign unique values to a tag variant.

```
tag HTTPCode {
    Ok: 202, // 202, duhh
    Okp1, // = 203 
    Forbidden: 403
    Error, // = 404
} 
```

to get the value of a tag as an integer, you can use the `discriminant()` method that each tag datatype gets automatically:

```Rust
code := HTTPCode:Ok;
code_as_int := code:discriminant();
if code_as_int == 202 {
    stdout:print-line("all fine!");
}
else {
    stdout:print-line("oopsie...");
}

// or:
if code == HTTPCode:Ok {
    stdout:print-line("all fine!");
}
else {
    stdout:print-line("oopsie...");
}
```

#### Unions
To define a unions, use the `union` keyword, field are declared the same way a struct does,
instantiating a union is done by treating the fields as contructor-functions:
```
union U64orF64 {
    uint: U64,
    float: F64
}

u := U64orF64:uint(3);
f := U64ofF64:float(3.1415);
```

To write to a union, you must select a variant to write to, and provide a value of that same type.

Unions are meant to be very primitive,
as such, there is no way to check which field was last written to or read from.
The size of a union is that of its largest variant, the above example would have a size of 64 bits.

Unions can be used to read bits of one type as if they were of another, this is called *type-punning*

See the below example, where the famous [Fast Inverse Square Root from Quake](en.wikipedia.org/wiki/Fast_inverse_square_root)
is implemented using type-punning.

```
union Helper {
    f: F64,
    u: U64,
}
fn Q-rsqrt(f: F64) -> F64 {
    x2 := f * 0.5;
    // construct as F64
    i := Helper:f(f);
    
    // then read as U64 and write to the U64 variant
    i.u = (0x5f3759df - (i.u >> 1));

    // then read and copy the value of the F64 variant, which is now modified 
    y := i.f; 

    y = y * (3.5 - (x2 * y * y)); // euler
    y // = (1 / sqrt(f))
}
```

#### Tagged unions
As of right now, tagged unions are a construct developers must build themselves, leading to lots of boilerplate.
I plan to implement tagged unions in a fashion similar to Rust somewhere in the future. Probably using some sort of syntax sugar.

### Namespaces
A file may declare a module, which contains imports, exports, datatypes, functions and globally-scoped identifiers.

```
// in file FooMod.zea
module FooMod

exports {
    fn foo;
    bar;
    struct Baz;
}

fn foo(a: U32) -> U32 {a * 2}
bar: U32 = 3;

struct Baz {
    baz_field: U32,
}
```
Which can then be imported in another module
```
// in file Chicken.zea
module Chicken

imports {
    FooMod:foo;
    FooMod:bar;
    FooMod:struct Baz;
}

struct Chicken {
    eggs_laid: U32,
}

fn Chicken:new() -> Chicken {
    Chicken {
        eggs-laid := 0
    }
}

fn Chicken:lay-egg(self: Chicken) {
    self.eggs_laid += 1;
    stdout:print-line("pock pock");
}

fn main(argv: [String]) -> U8 {
    let c := Chicken:new();
    c:lay-egg();
    c.eggs-laid
} 
```

The `:` operator has two uses, as a namespace-accessor, or alternatively as an method-accessor.
It is always known from context which one is meant, as Modules and Types always start with a capital letter,
while identifiers always start with a lowercase letter.
`Foo:bar()` (module-call) calls the `bar()` function defined in the `Foo` module.

`foo:bar()` (instance-call), in the case that `foo` is of type `Foo`,
calls a method defined in the implementation of type `Foo`, passing the variable `foo` as the first argument.
This is similar to Rust's `&self` parameter.

In the case of an instance-call, Fudge will infer which function to call by infering the type of the instance variable.
You may then notice there are now two ways to operate on instances of types:
```
module Foo
exports {struct Foo;}

struct Foo {}

fn bar(foo: Foo) {...}

foo := Foo {}; // instantiate some instance of the Foo type

// we can do an instance-call:
foo:bar();

// or, alternatively a module-call:
Foo:bar(foo);
```
the use of instance-calls is encouraged.

## Types
Fudge approaches specifying types in a manner more similar to functional Rust, where types have no modifiers,
instead a 'modified' type is considered a separate type.

Take for instance the `unsigned` modifier in C, compared to the `I32` vs. `U32` types in Rust.
This is done to make both the grammar of the language less ambiguous, and to make the code easier to parse for humans.

Fudge uses the `$var: $Type = $value` syntax instead of the `$Type $var = $value` syntax. This makes omitting types more pretty: `$var := $value`.

As you might have inferred (hah, get it) from the above, Fudge features type inference by a simplified Hindley Milner type system, Fudge features inference without generics (yet?),
which makes the type-checker simple (thank god).

## Simple linkage rules
As of right now, Fudge provides the `exports ()` block to specify public items,
other items are private. This is likely to change in the future.

The compiler does its best to ensure no namespace collisions occur, so you should be able to name your symbols whatever you want!

## backend
Fudge currently uses QBE as its backend, this is because it is easier to target than LLVM due to its simple syntax and ergonomics.
I wanted to hack on QBE's code such that it could be called as a library,
by constructing the IR using function calls instead of parsing a text file.
However, QBE's source code is quite hard to read. As both a learning excercise and an effort to the community,
I started work in an alternative implementation of QBE, this time in Rust with some added ergonomics.
You can view my progress [here](https://github.com/SleighPenguin22/rqbe)
