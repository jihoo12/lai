# alisp Documentation

alisp is a tiny Lisp interpreter written in Rust with zero dependencies. It is designed specifically for AI agents — providing shell execution, file I/O, HTTP requests, JSON handling, and string manipulation in a minimal, easy-to-generate language.

---

## Table of Contents

- [Installation](#installation)
- [Use as a Rust Library](#use-as-a-rust-library)
- [Usage](#usage)
- [Syntax](#syntax)
- [Data Types](#data-types)
- [Special Forms](#special-forms)
- [Built-in Functions](#built-in-functions)
  - [Shell Execution](#shell-execution)
  - [File I/O](#file-io)
  - [Environment Variables](#environment-variables)
  - [String Operations](#string-operations)
  - [List Operations](#list-operations)
  - [Arithmetic](#arithmetic)
  - [Comparison](#comparison)
  - [Logic](#logic)
  - [Type Operations](#type-operations)
  - [IO](#io)
  - [HTTP](#http)
  - [JSON](#json)
  - [Misc](#misc)
- [Error Handling](#error-handling)
- [Examples](#examples)
- [AI Agent Patterns](#ai-agent-patterns)

---

## Installation

```bash
cargo build --release
# Binary at: target/release/alisp
```

---

## Use as a Rust Library

alisp works both as a standalone CLI tool and as a Rust library you can embed in your own projects.

### Add Dependency

```toml
[dependencies]
alisp = { path = "../alisp" }
```

### Public API

```rust
use alisp::{Evaluator, Expr, expr_to_string, json_parse_str, json_stringify};

// Create an evaluator
let mut eval = Evaluator::new();

// Evaluate a single expression
let result = eval.eval_str("(+ 1 2)").unwrap().unwrap();
assert_eq!(expr_to_string(&result), "3");

// Evaluate multiline code (last expression's value is returned)
let result = eval.eval_str(r#"
    (def x 10)
    (def y 20)
    (+ x y)
"#).unwrap().unwrap();
assert_eq!(expr_to_string(&result), "30");

// Execute a file
eval.eval_file("script.lisp").unwrap();

// Access variables
eval.set_global("my_var".into(), alisp::string("hello"));
let val = eval.get("my_var").unwrap();
```

### Build AST Programmatically

You can construct alisp expressions directly from Rust:

```rust
use alisp::{num, string, sym, list, nil, bool_val, Expr};

// Build: (def x 42)
let expr = list(vec![
    sym("def"),
    sym("x"),
    num(42.0),
]);

// Build: (list 1 "hello" true)
let expr = list(vec![
    sym("list"),
    num(1.0),
    string("hello"),
    bool_val(true),
]);
```

### Parse and Stringify JSON

```rust
use alisp::{json_parse_str, json_stringify};

let data = json_parse_str(r#"{"name": "alisp", "tags": ["lisp", "ai"]}"#).unwrap();
let json = json_stringify(&data, false);   // pretty-printed
let compact = json_stringify(&data, true); // compact
```

### Available Types and Functions

| Export | Description |
|--------|-------------|
| `Evaluator` | The main interpreter. Call `eval_str()`, `eval()`, `eval_file()` |
| `Expr` | The AST/expression enum. Variants: `Num`, `Str`, `Bool`, `Nil`, `Sym`, `List`, `Lambda`, `Builtin` |
| `Token` | Tokenizer output |
| `tokenize(input)` | Tokenize a string into tokens |
| `parse(input)` | Parse a string into a list of expressions |
| `parse_first(input)` | Parse a single expression |
| `expr_to_string(expr)` | Convert an expression to its string representation |
| `expr_to_num(expr)` | Convert an expression to f64 |
| `expr_eq(a, b)` | Deep equality check |
| `is_truthy(expr)` | Check if an expression is truthy |
| `count_parens(s)` | Count unmatched parens (for multiline input) |
| `json_parse_str(s)` | Parse a JSON string |
| `json_stringify(expr, compact)` | Serialize to JSON |
| `num(n)`, `int(n)`, `string(s)`, `sym(s)`, `list(v)`, `nil()`, `bool_val(b)` | Construct AST nodes |
| `VERSION` | Version string constant |

No external dependencies. Requires Rust toolchain to build.

---

## Usage

```bash
# REPL mode (interactive)
alisp

# Execute a file
alisp script.lisp

# Execute a code string
alisp -e '(println "hello")'
```

### REPL

The REPL supports multiline input — if you open a parenthesis and press Enter, it will prompt for more:

```
alisp> (defn greet
  ...   (name)
  ...   (println "Hello," name))
nil
alisp> (greet "world")
Hello, world
```

Type `(exit)` or press `Ctrl+D` to quit.

---

## Syntax

alisp uses standard S-expression syntax.

### Atoms

```lisp
42          ; number
3.14        ; float
"hello"     ; string
true        ; boolean (also: t)
false       ; boolean (also: f)
nil         ; null (also: null or f)
my-var      ; symbol
```

### Lists

```lisp
(1 2 3)           ; list of numbers
(+ 1 2)           ; function call
(def x 10)        ; special form
```

### Comments

```lisp
; This is a line comment
```

### String Escapes

| Escape | Meaning |
|--------|---------|
| `\n` | newline |
| `\t` | tab |
| `\r` | carriage return |
| `\\` | backslash |
| `\"` | double quote |
| `\0` | null byte |

---

## Data Types

| Type | Examples | Description |
|------|----------|-------------|
| **number** | `42`, `3.14`, `-7` | 64-bit floating point |
| **string** | `"hello"` | UTF-8 string |
| **bool** | `true`, `false` | Boolean |
| **nil** | `nil`, `null` | Null/absence of value |
| **list** | `(1 2 3)` | Linked list of values |
| **function** | `(fn (x) x)` | Lambda/function |
| **symbol** | `foo`, `+`, `my-var` | Identifier |

Numbers are displayed as integers when possible (`42` instead of `42.0`).

Lists of pairs `(("key" val) ...)` are used to represent JSON objects.

---

## Special Forms

Special forms are evaluated differently from regular function calls — arguments may not be evaluated, or evaluation may be short-circuited.

### `def`

Define a global variable.

```lisp
(def x 10)           ; x = 10
(def greeting "hi")  ; greeting = "hi"
(def lst (list 1 2 3))
```

### `set!`

Mutate an existing variable. Looks in local scopes first, then global.

```lisp
(def x 1)
(set! x 2)  ; x is now 2

; Useful in loops
(def i 0)
(while (< i 5)
  (set! i (+ i 1)))
```

### `fn` / `lambda`

Create an anonymous function.

```lisp
(def double (fn (x) (* x 2)))
(double 5)  ; => 10

; Multiple parameters and body expressions
(def add-and-log (fn (a b)
  (let ((sum (+ a b)))
    (println "sum =" sum)
    sum)))
```

### `defn`

Define a named function (syntactic sugar for `def` + `fn`).

```lisp
(defn factorial (n)
  (if (<= n 1) 1 (* n (factorial (- n 1)))))

(factorial 5)  ; => 120
```

### `if`

Conditional expression.

```lisp
(if (> x 10)
  (println "big")
  (println "small"))

; else branch is optional
(if (nil? x) (println "x is nil"))
```

### `when`

Execute body when condition is true, otherwise nil.

```lisp
(when (= x 10)
  (println "x is ten")
  (do-something))
```

### `unless`

Execute body when condition is false (opposite of `when`).

```lisp
(unless (nil? x)
  (println "x is not nil"))
```

### `cond`

Multi-branch conditional.

```lisp
(cond
  ((< x 0)  (println "negative"))
  ((= x 0)  (println "zero"))
  (else     (println "positive")))
```

The last clause can be `(expression)` without a condition — it acts as a default.

### `do` / `begin`

Execute multiple expressions, return the last result.

```lisp
(do
  (def x 1)
  (def y 2)
  (+ x y))  ; => 3
```

### `let`

Local variable bindings.

```lisp
(let ((x 10) (y 20))
  (println "sum =" (+ x y)))

; Body can have multiple expressions
(let ((name "alisp") (ver "0.1.0"))
  (print name)
  (print " v")
  (println ver))
```

### `while`

Loop with a condition.

```lisp
(def i 0)
(while (< i 10)
  (println i)
  (set! i (+ i 1)))
```

### `dolist`

Iterate over a list.

```lisp
(dolist (x (list 1 2 3 4 5))
  (println "item:" x))

; With string list
(dolist (f (split "a,b,c" ","))
  (println f))
```

### `dotimes`

Iterate a number of times.

```lisp
(dotimes (i 5)
  (println i))
; Prints: 0 1 2 3 4

; Accumulate results
(def total 0)
(dotimes (i 10)
  (set! total (+ total i)))
; total = 45
```

### `quote`

Return the expression without evaluating it.

```lisp
(quote (1 2 3))  ; => (1 2 3), not evaluated
```

### `and`

Short-circuit logical AND. Returns the first falsy value, or the last value if all truthy.

```lisp
(and true true true)    ; => true
(and true false true)   ; => false
(and "hello" 42)        ; => 42
(and nil "nope")        ; => nil
```

### `or`

Short-circuit logical OR. Returns the first truthy value, or the last value if all falsy.

```lisp
(or false nil "yes")    ; => "yes"
(or false nil)          ; => nil
(or 1 2 3)              ; => 1
```

### `try` / `catch`

Error handling. If an error occurs in the `try` body, execution jumps to `catch`.

```lisp
(try
  (exec "command-that-might-fail")
  (catch e
    (println "Error:" e)))

; catch variable holds the error message as a string
(try
  (json-parse "not json")
  (catch e
    (println "Parse error:" e)))
```

### `throw`

Raise an error with a message.

```lisp
(defn divide (a b)
  (if (= b 0)
    (throw "division by zero")
    (/ a b)))

(try
  (divide 10 0)
  (catch e (println e)))
; => "division by zero"
```

### `apply`

Apply a function to a list of arguments.

```lisp
(apply + (list 1 2 3))     ; => 6
(apply max (list 5 3 8))   ; => 8
```

### `eval`

Evaluate a string as alisp code.

```lisp
(eval "(+ 1 2)")           ; => 3
(def code "(list 1 2 3)")
(eval code)                 ; => (1 2 3)
```

---

## Built-in Functions

### Shell Execution

#### `exec` (aliases: `shell`, `sh`)

Run a shell command and return stdout as a trimmed string. Returns an error if the command fails (non-zero exit code).

```lisp
(exec "whoami")              ; => "jihoo"
(exec "date +%Y-%m-%d")      ; => "2026-07-13"
(exec "echo hello && echo world")  ; => "hello\nworld"

; Multiple arguments are joined with spaces
(exec "git" "log" "--oneline" "-5")

; Error handling
(try
  (exec "ls /nonexistent")
  (catch e (println "Failed:" e)))
```

#### `exec-result`

Run a shell command and return a structured result: `((status N) (stdout "...") (stderr "..."))`.

```lisp
(let ((result (exec-result "echo hello && false")))
  (let ((status  (nth (nth result 0) 1))
        (stdout  (nth (nth result 1) 1))
        (stderr  (nth (nth result 2) 1)))
    (println "exit:" status)
    (println "out:" stdout)
    (println "err:" stderr)))
```

---

### File I/O

#### `read` / `read-file`

Read entire file contents as a string.

```lisp
(def content (read "Cargo.toml"))
(println content)
```

#### `read-lines`

Read a file and return its contents as a list of lines (without newline characters).

```lisp
(def lines (read-lines "Cargo.toml"))
(println (len lines))       ; number of lines
(println (car lines))       ; first line
(dolist (line lines)
  (println ">" line))
```

#### `write` / `write-file`

Write a string to a file (creates or overwrites).

```lisp
(write "output.txt" "hello world\n")
(write "data.json" (json-stringify my-data))
```

#### `append` / `append-file`

Append a string to a file (creates if doesn't exist).

```lisp
(append "log.txt" "new log entry\n")
```

#### `exists` / `file-exists`

Check if a file or directory exists.

```lisp
(exists "Cargo.toml")   ; => true
(exists "/nope")         ; => false
```

#### `rm` / `delete`

Remove a file or directory (recursive for directories).

```lisp
(rm "temp.txt")
(rm "old-directory")
```

#### `mkdir`

Create a directory (recursive, like `mkdir -p`).

```lisp
(mkdir "path/to/nested/dir")
```

#### `touch`

Create an empty file or update an existing file's timestamp.

```lisp
(touch "new-file.txt")       ; creates empty file
(touch "existing.log")       ; updates timestamp
```

#### `file-size`

Get the size of a file in bytes.

```lisp
(file-size "Cargo.toml")     ; => 1542
```

#### `file?`

Check if a path is a file (not a directory).

```lisp
(file? "Cargo.toml")  ; => true
(file? "src")          ; => false
```

#### `dir?`

Check if a path is a directory.

```lisp
(dir? "src")           ; => true
(dir? "Cargo.toml")    ; => false
```

#### `mtime`

Get the last modification time of a file as a Unix timestamp (seconds since epoch).

```lisp
(mtime "Cargo.toml")   ; => 1783926780
```

#### `cp` / `copy`

Copy a file or directory. Directories are copied recursively.

```lisp
(cp "source.txt" "backup.txt")
(cp "src-dir" "dst-dir")  ; recursive copy
```

#### `mv` / `move`

Move/rename a file.

```lisp
(mv "old-name.txt" "new-name.txt")
```

#### `ls` / `list-dir`

List directory contents, returns sorted list of filenames.

```lisp
(ls)                ; list current directory
(ls "/tmp")         ; list /tmp
(ls "src")          ; list src/
```

#### `glob`

Find files matching a glob pattern. Supports `*`, `**`, and `?` wildcards.

| Pattern | Description |
|---------|-------------|
| `*.rs` | Match files in current directory |
| `src/**/*.rs` | Match all `.rs` files recursively under `src/` |
| `*.?` | Match single character wildcard |
| `**/*.toml` | Match all `.toml` files anywhere |

```lisp
(glob "*.rs")           ; Rust files in current dir
(glob "src/**/*.rs")    ; all .rs files under src/ (recursive)
(glob "data/*.csv")     ; CSV files in data/
(glob "file?.txt")      ; file1.txt, file2.txt, etc.
```

#### `cwd` / `pwd`

Get current working directory.

```lisp
(cwd)  ; => "/home/user/project"
```

#### `cd`

Change current working directory.

```lisp
(cd "/tmp")
(println (cwd))  ; => "/tmp"
```

#### `basename`

Get the filename from a path. Optionally strip a suffix.

```lisp
(basename "src/main.rs")          ; => "main.rs"
(basename "src/main.rs" ".rs")    ; => "main"
(basename "/home/user/file.txt")  ; => "file.txt"
```

#### `dirname`

Get the directory portion of a path.

```lisp
(dirname "src/main.rs")          ; => "src"
(dirname "/home/user/file.txt")  ; => "/home/user"
```

#### `ext`

Get the file extension (without the dot).

```lisp
(ext "main.rs")          ; => "rs"
(ext "archive.tar.gz")   ; => "gz"
(ext "Makefile")         ; => ""
```

#### `join-path`

Join path segments into a single path.

```lisp
(join-path "src" "lib" "mod.rs")  ; => "src/lib/mod.rs"
(join-path "/home" "user")         ; => "/home/user"
```

#### `realpath`

Resolve a path to its absolute canonical form (resolves `..`, `.`, and symlinks).

```lisp
(realpath ".")              ; => "/home/user/project"
(realpath "../src/../src")  ; => "/home/user/project/src"
```

#### `read-range`

Read lines from a file by line range (1-indexed, inclusive). Returns a list of line strings.

```lisp
(write "data.txt" "a\nb\nc\nd\ne")
(read-range "data.txt" 2 4)   ; => ("b" "c" "d")
(read-range "data.txt" 1 1)   ; => ("a")
```

#### `write-range`

Replace a range of lines (1-indexed, inclusive) with new content. Content can be a string (with newlines) or a list of strings.

```lisp
(write "data.txt" "a\nb\nc\nd\ne")
(write-range "data.txt" 2 3 "X\nY")   ; lines 2-3 replaced
(read "data.txt")                       ; => "a\nX\nY\nd\ne"

; Replace with a list
(write-range "data.txt" 1 3 (list "one" "two" "three"))
```

#### `insert-at`

Insert content before a given line number (1-indexed). Content can be a string (with newlines) or a list of strings.

```lisp
(write "data.txt" "a\nb\nc")
(insert-at "data.txt" 2 "NEW")   ; insert before line 2
(read "data.txt")                 ; => "a\nNEW\nb\nc"

; Insert multiple lines
(insert-at "data.txt" 1 (list "first" "second"))
```

#### `remove-range`

Remove a range of lines (1-indexed, inclusive).

```lisp
(write "data.txt" "a\nb\nc\nd\ne")
(remove-range "data.txt" 2 4)   ; remove lines 2-4
(read "data.txt")                ; => "a\ne"
```

---

### Environment Variables

#### `getenv` / `env-get`

Get an environment variable. Returns nil if not set.

```lisp
(getenv "HOME")    ; => "/home/jihoo"
(getenv "USER")    ; => "jihoo"
(getenv "NOPE")    ; => nil
```

#### `setenv` / `env-set`

Set an environment variable.

```lisp
(setenv "MY_VAR" "hello")
(println (getenv "MY_VAR"))  ; => "hello"
```

#### `env`

Get all environment variables as a list of `(key value)` pairs.

```lisp
(def vars (env))
; vars is (("HOME" "/home/jihoo") ("USER" "jihoo") ...)
```

---

### String Operations

#### `str`

Concatenate values into a string.

```lisp
(str "hello " "world")           ; => "hello world"
(str "count: " 42)               ; => "count: 42"
(str (list 1 2 3))               ; => "(1 2 3)"
```

#### `split`

Split a string by a delimiter.

```lisp
(split "a,b,c,d" ",")     ; => ("a" "b" "c" "d")
(split "hello world" " ")  ; => ("hello" "world")
(split "abc" ",")          ; => ("abc")
```

#### `join`

Join a list of strings with a delimiter.

```lisp
(join (list "a" "b" "c") ",")    ; => "a,b,c"
(join (list "hello" "world") " ") ; => "hello world"
```

#### `trim`

Remove leading/trailing whitespace.

```lisp
(trim "  hello  ")  ; => "hello"
```

#### `contains` / `includes`

Check if a string contains a substring.

```lisp
(contains "hello world" "world")  ; => true
(contains "hello" "xyz")          ; => false
```

#### `starts-with`

Check if a string starts with a prefix.

```lisp
(starts-with "hello" "hel")  ; => true
```

#### `ends-with`

Check if a string ends with a suffix.

```lisp
(ends-with "hello" "llo")  ; => true
```

#### `replace`

Replace first occurrence of a substring.

```lisp
(replace "hello world" "world" "alisp")  ; => "hello alisp"
```

#### `upper` / `lower`

Case conversion.

```lisp
(upper "hello")  ; => "HELLO"
(lower "HELLO")  ; => "hello"
```

#### `substr`

Extract a substring: `(substr string start length)`.

```lisp
(substr "hello world" 0 5)   ; => "hello"
(substr "hello world" 6 5)   ; => "world"
```

#### `find`

Find the index of a substring. Returns -1 if not found.

```lisp
(find "hello world" "world")  ; => 6
(find "hello" "xyz")          ; => -1
```

#### `format`

Python-style string formatting with `{}` placeholders.

```lisp
(format "Hello, {}!" "world")                    ; => "Hello, world!"
(format "{} + {} = {}" 2 3 5)                    ; => "2 + 3 = 5"
(format "{0} is {1}" "alisp" "awesome")          ; => "alisp is awesome"
```

---

### List Operations

#### `list`

Create a list from arguments.

```lisp
(list 1 2 3)         ; => (1 2 3)
(list "a" "b" "c")   ; => ("a" "b" "c")
(list)                ; => ()
```

#### `car` / `head` / `first`

Get the first element of a list or string.

```lisp
(car (list 1 2 3))   ; => 1
(car "hello")         ; => "h"
```

#### `cdr` / `tail` / `rest`

Get everything except the first element.

```lisp
(cdr (list 1 2 3))   ; => (2 3)
(cdr "hello")         ; => "ello"
```

#### `cons`

Prepend an element to a list.

```lisp
(cons 0 (list 1 2 3))  ; => (0 1 2 3)
(cons "a" (list))       ; => ("a")
```

#### `len` / `length` / `size`

Get the length of a list or string.

```lisp
(len (list 1 2 3))  ; => 3
(len "hello")        ; => 5
(len (list))          ; => 0
```

#### `push`

Append an element to a list (returns new list).

```lisp
(push (list 1 2) 3)  ; => (1 2 3)

; To accumulate, use set!
(def items (list))
(set! items (push items "a"))
(set! items (push items "b"))
; items => ("a" "b")
```

#### `nth` / `at`

Get element at index.

```lisp
(nth (list 10 20 30) 1)  ; => 20
(nth "hello" 2)           ; => "l"
(nth (list) 0)            ; => nil
```

#### `map`

Apply a function to each element.

```lisp
(map (fn (x) (* x 2)) (list 1 2 3))  ; => (2 4 6)
(map upper (list "hello" "world"))     ; => ("HELLO" "WORLD")
```

#### `filter` / `select`

Keep elements where the function returns truthy.

```lisp
(filter (fn (x) (> x 3)) (list 1 2 3 4 5))  ; => (4 5)
(filter (fn (x) (= (% x 2) 0)) (range 1 11)) ; => (2 4 6 8 10)
```

#### `reduce` / `fold`

Combine elements with a function and initial value.

```lisp
(reduce + 0 (list 1 2 3 4 5))  ; => 15
(reduce * 1 (list 2 3 4))       ; => 24
(reduce (fn (acc x) (str acc x)) "" (list "a" "b" "c"))  ; => "abc"
```

#### `each` / `for-each`

Call a function for side effects on each element.

```lisp
(each println (list 1 2 3))
; Prints:
; 1
; 2
; 3
```

#### `range`

Generate a list of numbers.

```lisp
(range 5)           ; => (0 1 2 3 4)
(range 1 6)         ; => (1 2 3 4 5)
(range 0 10 2)      ; => (0 2 4 6 8)
(range 10 0 -2)     ; => (10 8 6 4 2)
```

#### `reverse`

Reverse a list or string.

```lisp
(reverse (list 1 2 3))  ; => (3 2 1)
(reverse "hello")        ; => "olleh"
```

#### `sort`

Sort a list alphabetically/by string comparison.

```lisp
(sort (list 3 1 4 1 5))       ; => (1 1 3 4 5)
(sort (list "c" "a" "b"))     ; => ("a" "b" "c")
```

#### `flatten`

Flatten nested lists.

```lisp
(flatten (list 1 (list 2 3) (list (list 4 5))))  ; => (1 2 3 4 5)
```

#### `last`

Get the last element.

```lisp
(last (list 1 2 3))  ; => 3
(last "hello")        ; => "o"
```

#### `empty?`

Check if a list or string is empty.

```lisp
(empty? (list))  ; => true
(empty? "")       ; => true
(empty? (list 1)) ; => false
```

#### `any`

Check if any element satisfies a predicate.

```lisp
(any (fn (x) (> x 3)) (list 1 2 3 4 5))  ; => true
(any (fn (x) (> x 10)) (list 1 2 3))      ; => false
```

#### `all`

Check if all elements satisfy a predicate.

```lisp
(all (fn (x) (> x 0)) (list 1 2 3))  ; => true
(all (fn (x) (> x 2)) (list 1 2 3))  ; => false
```

#### `zip`

Combine multiple lists element-wise.

```lisp
(zip (list 1 2 3) (list "a" "b" "c"))  ; => ((1 "a") (2 "b") (3 "c"))
(zip (list 1 2) (list 3 4) (list 5 6))  ; => ((1 3 5) (2 4 6))
```

#### `assoc`

Add or update a key-value pair in an object (list of pairs).

```lisp
(def obj (list))
(def obj (assoc obj "name" "alisp"))
(def obj (assoc obj "ver" 1))
; obj => (("name" "alisp") ("ver" 1))
```

#### `dissoc`

Remove a key from an object.

```lisp
(dissoc (list ("name" "alisp") ("ver" 1)) "ver")
; => (("name" "alisp"))
```

#### `keys`

Get all keys from an object.

```lisp
(keys (json-parse "{\"a\":1,\"b\":2}"))  ; => ("a" "b")
```

#### `values`

Get all values from an object.

```lisp
(values (json-parse "{\"a\":1,\"b\":2}"))  ; => (1 2)
```

#### `merge`

Merge multiple lists/objects.

```lisp
(merge (list 1 2) (list 3 4))  ; => (1 2 3 4)
```

---

### Arithmetic

| Function | Description | Example |
|----------|-------------|---------|
| `+` | Add (also concatenates strings) | `(+ 1 2 3)` => `6` |
| `-` | Subtract (unary negate) | `(- 10 3)` => `7` |
| `*` | Multiply | `(* 2 3 4)` => `24` |
| `/` | Divide | `(/ 10 3)` => `3.333...` |
| `%` / `mod` | Modulo | `(% 10 3)` => `1` |
| `pow` | Power | `(pow 2 10)` => `1024` |
| `sqrt` | Square root | `(sqrt 9)` => `3` |
| `abs` | Absolute value | `(abs -5)` => `5` |
| `min` | Minimum | `(min 3 1 4)` => `1` |
| `max` | Maximum | `(max 3 1 4)` => `4` |
| `floor` | Round down | `(floor 3.7)` => `3` |
| `ceil` | Round up | `(ceil 3.2)` => `4` |
| `round` | Round to nearest | `(round 3.5)` => `4` |
| `rand` | Random 0-1 | `(rand)` => `0.732...` |
| `rand` N | Random 0-N (int) | `(rand 100)` => `42` |
| `inc` | Increment | `(inc 5)` => `6` |
| `dec` | Decrement | `(dec 5)` => `4` |

String `+` concatenates: `(+ "hello " "world")` => `"hello world"`

---

### Comparison

| Function | Description |
|----------|-------------|
| `=` / `==` | Equal |
| `!=` | Not equal |
| `<` | Less than |
| `>` | Greater than |
| `<=` | Less or equal |
| `>=` | Greater or equal |

```lisp
(= 1 1)           ; => true
(!= "a" "b")      ; => true
(< 3 5)            ; => true
(>= 10 10)         ; => true
```

---

### Logic

| Function | Description |
|----------|-------------|
| `and` | Short-circuit AND (special form) |
| `or` | Short-circuit OR (special form) |
| `not` | Logical negation |

```lisp
(and true "yes")   ; => "yes"
(and nil "no")     ; => nil
(or false "yes")   ; => "yes"
(or nil nil)       ; => nil
(not nil)          ; => true
(not 42)           ; => false
```

**Truthy values:** everything except `nil`, `false`, `0`, `""`, and `()`.

---

### Type Operations

| Function | Description |
|----------|-------------|
| `type` / `type-of` | Get type name as string |
| `int` | Convert to integer |
| `float` | Convert to float |
| `number?` | Check if number |
| `string?` | Check if string |
| `list?` | Check if list |
| `nil?` | Check if nil |
| `bool?` | Check if boolean |

```lisp
(type 42)         ; => "number"
(type "hi")       ; => "string"
(type (list))     ; => "list"
(type nil)        ; => "nil"
(type true)       ; => "bool"

(int "42")        ; => 42
(float "3.14")    ; => 3.14
(int 3.9)         ; => 3
```

---

### IO

#### `print`

Print values without a newline. Values are joined with spaces.

```lisp
(print "hello" "world")  ; prints: hello world
```

#### `println`

Print values with a newline.

```lisp
(println "x =" 42)  ; prints: x = 42\n
```

#### `eprint` / `eprintln`

Print to stderr.

```lisp
(eprintln "warning:" msg)
```

#### `input`

Read a line from stdin with an optional prompt.

```lisp
(def name (input "Enter name: "))
```

---

### HTTP

All HTTP functions use `curl` under the hood. Requires `curl` to be installed.

#### `http-get`

```lisp
(def body (http-get "https://httpbin.org/get"))
(println body)
```

#### `http-post`

```lisp
(def response (http-post "https://httpbin.org/post" "{\"key\":\"value\"}"))
```

#### `http-put`

```lisp
(http-put "https://api.example.com/resource/1" "{\"name\":\"updated\"}")
```

#### `http-delete`

```lisp
(http-delete "https://api.example.com/resource/1")
```

#### `http` (full control)

```lisp
(http "GET" "https://httpbin.org/get")
(http "POST" "https://httpbin.org/post" "{\"data\":1}")
(http "POST" url body '("Content-Type: application/json" "Authorization: Bearer token"))
```

---

### JSON

#### `json-parse` / `json`

Parse a JSON string into alisp data structures.

| JSON | alisp |
|------|-------|
| `null` | `nil` |
| `true`/`false` | `true`/`false` |
| `42` | `42` |
| `"hello"` | `"hello"` |
| `[1,2,3]` | `(1 2 3)` |
| `{"a":1}` | `(("a" 1))` |

```lisp
(def data (json-parse "{\"name\":\"alisp\",\"tags\":[\"lisp\",\"ai\"]}"))
(println (json-get data "name"))    ; => "alisp"
(println (json-get data "tags"))    ; => ("lisp" "ai")
```

#### `json-stringify` / `json-str`

Convert alisp data to JSON string.

```lisp
(json-stringify (list 1 2 3))                     ; => "[\n  1,\n  2,\n  3\n]"
(json-stringify (json-parse "{\"a\":1}"))          ; pretty-printed
(json-stringify (json-parse "{\"a\":1}") "compact") ; compact: {"a":1}
```

#### `json-get` / `jget`

Get a value from a JSON object or array.

```lisp
(json-get (list 10 20 30) 1)           ; => 20  (array index)
(json-get (json-parse "{\"x\":1}") "x") ; => 1  (object key)
```

#### `json-set` / `jset`

Set a value in a JSON object (returns new object).

```lisp
(def obj (json-parse "{}"))
(def obj (json-set obj "name" "alisp"))
(def obj (json-set obj "version" 1))
; obj => (("name" "alisp") ("version" 1))
```

#### `json-keys`

Get keys from a JSON object.

```lisp
(json-keys (json-parse "{\"a\":1,\"b\":2}"))  ; => ("a" "b")
```

---

### Regular Expressions

alisp includes a built-in regex engine supporting the following syntax:

| Pattern | Description |
|---------|-------------|
| `abc` | Literal characters |
| `.` | Any character |
| `\d` / `\D` | Digit / non-digit |
| `\w` / `\W` | Word char (alphanumeric + `_`) / non-word |
| `\s` / `\S` | Whitespace / non-whitespace |
| `[abc]` | Character class |
| `[a-z]` | Character range |
| `[^abc]` | Negated class |
| `*` | Zero or more |
| `+` | One or more |
| `?` | Zero or one |
| `\|` | Alternation |
| `(...)` | Group |
| `^` | Start of string anchor |
| `$` | End of string anchor |

#### `re-test` / `re-match?`

Test if a regex pattern matches anywhere in a string. Returns `true` or `false`.

```lisp
(re-test "hello.*world" "hello beautiful world")  ; => true
(re-test "^[a-z]+$" "hello")                       ; => true
(re-test "^[a-z]+$" "hello123")                    ; => false
(re-test "\\d+" "abc123")                          ; => true
```

#### `re-match`

Test if a regex pattern matches the **entire** string. Returns a match result list `(match start end)` or `nil`.

```lisp
(re-match "^[a-z]+$" "hello")     ; => ("hello" 0 5)
(re-match "^[a-z]+$" "abc123")    ; => nil
(re-match "\\d+" "123abc")        ; => nil (doesn't match full string)
```

#### `re-find`

Find the first occurrence of a pattern in a string. Returns the matched string or `nil`.

```lisp
(re-find "\\d+" "abc123def456")   ; => "123"
(re-find "http" "visit http://example.com")  ; => "http"
(re-find "[a-z]+" "ABC123xyz")    ; => "xyz"
```

#### `re-find-all`

Find all non-overlapping matches. Returns a list of matched strings.

```lisp
(re-find-all "\\d+" "abc123def456ghi789")  ; => ("123" "456" "789")
(re-find-all "[aeiou]+" "hello world")      ; => ("e" "o" "o")
```

#### `re-replace`

Replace the **first** occurrence of a pattern.

```lisp
(re-replace "hello world" "world" "alisp")  ; => "hello alisp"
(re-replace "aaa" "a+" "X")                 ; => "Xa" (only first match)
```

#### `re-replace-all`

Replace **all** occurrences of a pattern.

```lisp
(re-replace-all "aabbcc" "b+" "X")  ; => "aaXcc"
(re-replace-all "aaa" "a+" "X")     ; => "X" (all 'a' sequences replaced)
```

#### `re-split`

Split a string by a regex pattern. Returns a list of substrings.

```lisp
(re-split "\\s+" "hello world foo bar")  ; => ("hello" "world" "foo" "bar")
(re-split "," "one,two,three")           ; => ("one" "two" "three")
(re-split "\\d+" "abc123def456")         ; => ("abc" "def" "")
```

#### `re-scan`

Find all matches with their positions. Returns a list of `(match start end)` tuples.

```lisp
(re-scan "[a-z]+" "abc 123 def 456")
; => (("abc" 0 3) ("def" 8 11))
```

#### Regex Examples

```lisp
; Validate email-like pattern
(re-test "^[a-z]+@[a-z]+\\.[a-z]+$" "user@example.com")  ; => true

; Extract all numbers from a string
(re-find-all "-?\\d+\\.?\\d*" "pi is 3.14 and e is 2.71")
; => ("3.14" "2.71")

; Normalize whitespace
(re-replace-all "\\s+" "hello    world   foo" " ")
; => "hello world foo"

; Extract domains from URLs
(def urls (list "https://example.com/path" "http://test.org/page"))
(map (fn (url) (re-find "https?://([^/]+)" url)) urls)
```

---

### Misc

#### `sleep`

Pause execution for N seconds (supports fractional seconds).

```lisp
(sleep 1)        ; wait 1 second
(sleep 0.5)      ; wait 500ms
```

#### `time`

Get elapsed time since interpreter started (seconds).

```lisp
(println "elapsed:" (time))
```

#### `timestamp`

Get Unix timestamp (seconds since epoch).

```lisp
(println "now:" (timestamp))
```

#### `exit` / `quit`

Exit the interpreter.

```lisp
(exit)      ; exit with code 0
(exit 1)    ; exit with code 1
```

---

## Error Handling

alisp uses `try`/`catch` for error handling. Errors are caught as strings.

```lisp
; Basic pattern
(try
  (dangerous-operation)
  (catch error-message
    (println "Error:" error-message)))

; Nested error handling
(defn safe-divide (a b)
  (try
    (/ a b)
    (catch e
      (println "Division error:" e)
      nil)))

(safe-divide 10 0)  ; prints "Division error: Division by zero", returns nil
```

Errors from `exec`, `read`, `write`, `http-get`, and other I/O operations are automatically catchable.

---

## Examples

### System Info Script

```lisp
(println "=== System Info ===")
(println "Hostname:" (exec "hostname"))
(println "OS:" (exec "uname -s"))
(println "Arch:" (exec "uname -m"))
(println "User:" (getenv "USER"))
(println "Home:" (getenv "HOME"))
(println "Date:" (exec "date"))
```

### File Processing

```lisp
(defn count-words (filepath)
  (let ((content (read filepath))
        (words (split content " ")))
    (len words)))

(println "Words in Cargo.toml:" (count-words "Cargo.toml"))
```

### JSON API Pipeline

```lisp
; Fetch, parse, transform, save
(def raw (http-get "https://httpbin.org/get"))
(def data (json-parse raw))
(def origin (json-get data "origin"))
(def report (json-parse "{}"))
(def report (json-set report "origin" origin))
(def report (json-set report "timestamp" (timestamp)))
(write "api-report.json" (json-stringify report))
(println "Report saved!")
```

### Batch File Processing

```lisp
(def files (glob "*.rs"))
(println "Found" (len files) "Rust files")
(dolist (f files)
  (let ((content (read f))
        (lines (split content "\n")))
    (println f ":" (len lines) "lines")))
```

### Simple Web Scraper

```lisp
(def html (http-get "https://example.com"))
(def title-start (find html "<title>"))
(def title-end (find html "</title>"))
(when (and (> title-start -1) (> title-end -1))
  (let ((title (substr html (+ title-start 8) (- title-end title-start 8))))
    (println "Page title:" (trim title))))
```

---

## AI Agent Patterns

### Error Recovery

Always wrap risky operations in try/catch:

```lisp
(defn safe-exec (cmd)
  (try
    (exec cmd)
    (catch e "")))

(def result (safe-exec "curl -s https://api.example.com"))
```

### Structured Output

Build JSON for downstream consumption:

```lisp
(defn make-report (task status details)
  (json-stringify
    (json-parse
      (format "{{\"task\":\"{}\",\"status\":\"{}\",\"details\":\"{}\"}}"
        task status details))))
```

### Composable Pipeline

Chain operations with `let`:

```lisp
(let ((raw (exec "ps aux"))
      (lines (split raw "\n"))
      (header (car lines))
      (procs (cdr lines)))
  (println "Processes:" (- (len lines) 1)))
```

### Retry Pattern

```lisp
(defn retry (n fn)
  (if (<= n 0)
    (throw "max retries exceeded")
    (try
      (fn)
      (catch e
        (println "retry" (- n 1) ":" e)
        (sleep 1)
        (retry (- n 1) fn)))))

(retry 3 (fn () (exec "curl -s https://httpbin.org/status/200")))
```
