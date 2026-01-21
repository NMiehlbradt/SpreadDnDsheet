# SpreadDnDsheet

**A reactive spreadsheet engine purpose-built for the complexity of Dungeons & Dragons.**

SpreadDnD Sheet is a spreadsheet engine designed for Dungeons and Dragons character sheets. D&D character sheets are basically just spreadsheets, you have a bunch of stats which all depend on each other in some way. However, D&D character features are often expressed as how they impact character stats and abilties, and the open-ended nature of the game means that a it is impossible to know in advance all the things that may affect some piece of data.

SpreadDnD Sheet also allows for cell formulas to affect other cells in a structured way to allow for easier translation from game rules to the formula language.

## Formula Quick Start Guide

The SpreadDnDsheet formula language is a functional, expression-based language designed to handle complex character data. It supports lexical scoping, first-class functions, and powerful data manipulation tools.
1. Basic Types & Arithmetic

The language supports standard integers, booleans, and basic mathematical operations.

- Math: `+, -, *`
- Logic: `true, false, and, or, not`

```
(10 + 5) * 2  -- Result: 30
```

2. Data Structures

You can organize data using Lists and Records.

Lists: Ordered collections, e.g., [1, 2, 3].

Records: Key-value pairs (like JSON objects or D&D stat blocks).

Indexing: Access elements using a dot or bracket notation.

```
let stats = { strength: 15, dexterity: 12 } in
stats.strength  -- Result: 15
```

3. Record Updates (// Operator)

Merges two records. If a key exists in both, the value from the right operand (the "patch") wins.

```
let 
    base_stats = { strength: 10, dexterity: 10 };
    bonus = { strength: 12 }
in

base_stats // bonus  -- Result: { strength: 12, dexterity: 10 }
```

4. Functional Tools (map, fold, filter)

These functions are overloaded to work on both Lists and Records.

`map(fn, data)`: Transforms every element.

`filter(fn, data)`: Keeps elements that satisfy a condition.

`fold(fn, initial, data)`: Reduces a collection to a single value.

5. Scoping & Functions

The language uses `let ... in` syntax for local variables. You can define anonymous functions (lambdas) that capture their surrounding scope.

Note: Functions are first-class citizens, but recursive definitions are not supported to ensure the spreadsheet remains a Directed Acyclic Graph (DAG) and avoids infinite loops.

```
let multiply_by = fn (x) -> fn (y) -> x * y in
let double = multiply_by(2) in
double(10)  -- Result: 20
```

6. Writing values to other cells

There are two special builtin functions that allow cells to send data to other cells. `push` takes a name of a cell and a value. That value is inserted into a list that can be read by the target cell with `read()`. `push` also returns the value that was pushed.

```
# Cell A
push("C", 10) -- Result: 10

# Cell B
push("C", "Hello") -- Result: "Hello"

# Cell C
read() -- Result [10, "Hello"]
```

Any value can be pushed to any other cell as long as it doesn't create a dependancy cycle. Cells can push to multiple different cells or the same cell multiple times. When pushed values are read they are returned in alphabetical order by cell name, with pushes from the same cell occuring in the order they were evaluated.

## Building & Installation
### Prerequisites

- Stable Rust Toolchain: Ensure you have the latest stable version of Rust installed via rustup.
- Nix (Optional): If you use Nix, a flake.nix is provided to set up your environment automatically.

### Native Build

To run the desktop GUI version:

```
# Clone the repository
git clone https://github.com/your-username/SpreadDnDsheet.git
cd SpreadDnDsheet

# Run the iced frontend
cargo run --release
```

### WebAssembly (WASM) Build

SpreadDnDsheet can be compiled to WASM for web integration.

```
# Add the WASM target
rustup target add wasm32-unknown-unknown

# Build the project for web
cargo build --target wasm32-unknown-unknown --release
```

### Using Nix

If you have Nix installed with Flakes enabled, you can enter a development shell with all dependencies (including Rust and system libraries for Iced) pre-configured:

```
nix develop
# Then run as usual
cargo run
```