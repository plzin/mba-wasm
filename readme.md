# Mixed Boolean-Arithmetic Obfuscation
The algorithm transforms expressions like `x+y` into things like this:
```c
-38*(x & y) - 83*(x ^ y) - 64*~(x ^ (y ^ z)) - 41*~x - 43*~y - 23*y - 44*z - 20*(y & z) - 21*(x | z) - 107*(~x & z) - 108*(y | ~z)
```
These kind of expressions involving both normal arithmetic as well as boolean operations are known as mixed boolean-arithmetic expressions.
This particular transformation is only valid when `x` and `y` (and `z`) are 8-bit integers and the usual rules of computer arithmetic apply (e.g. when adding/multiplying numbers and there is an overflow then the most significant bits that can not be represented are cut off).
In particular this will not work when the numbers are floating point numbers.
Rust itself will panic (at least in debug builds) when addition/multiplication overflows so in order to use this with rust you will have to use the [Wrapping](https://doc.rust-lang.org/std/num/struct.Wrapping.html) types.

### Usage
There is a [web interface](https://plzin.github.io/mba-wasm/).

Generating linear MBA expressions involves solving systems of linear congruences
which can be done [here](https://plzin.github.io/mba-wasm/linear_congruences.html).
This was mostly used during debugging but hopefully someone can find use for this.

### How it works
If you want to understand the algorithm, check out my [blog post](https://plzin.github.io/posts/mba) about it.
The algorithm is implemented in Rust and compiles to WebAssembly, that will be run in your browser.

### To Do
Currently only linear MBA expressions are implemented.
The permutation polynomial part is missing.