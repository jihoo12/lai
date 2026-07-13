# alisp Language Reference

S-expression Lisp dialect. All code is a sequence of expressions evaluated in order.

## Types

| Type | Literals | Notes |
|------|----------|-------|
| number | `42`, `3.14`, `-7` | 64-bit float, displayed as int when possible |
| string | `"hello"` | Double-quoted, escapes: `\n \t \r \\ \" \0` |
| bool | `true`, `false` | Also: `t` for true |
| nil | `nil`, `null` | Falsy. Also `false`, `0`, `""`, `()` are falsy |
| list | `(1 2 3)` | Heterogeneous, can nest |
| function | `(fn (x) x)` | Lambda with closure |
| symbol | `foo`, `my-var` | Bare identifiers |

Lists of 2-element lists with string first element represent JSON objects: `(("key" val) ...)`

## Comments

```lisp
; line comment
```

## Special Forms

These do NOT evaluate all arguments before execution.

```
(def name value)                          ; define global
(set! name value)                         ; mutate (local or global)
(fn (params...) body...)                  ; anonymous function
(defn name (params...) body...)           ; named function

(if cond then else?)                      ; conditional
(when cond body...)                       ; if-true block
(unless cond body...)                     ; if-false block
(cond (test expr)... (expr))              ; multi-branch, last can be bare default

(do body...)                              ; sequential, returns last
(let ((name val)...) body...)             ; local bindings
(while cond body...)                      ; loop
(dolist (var list-expr) body...)          ; iterate list
(dotimes (var n-expr) body...)            ; iterate 0..n-1

(and expr...)                             ; short-circuit AND
(or expr...)                              ; short-circuit OR

(try body... (catch var handler...))      ; error handling
(throw expr)                              ; raise error

(apply func list-expr)                    ; call with list of args
(eval string-expr)                        ; eval string as code
(quote expr)                              ; return unevaluated
```

## Built-in Functions

### Shell

```
(exec "cmd")                 ; run shell command, return stdout string (error on fail)
(exec "arg1" "arg2")         ; args joined with spaces, run in sh -c
(exec-result "cmd")          ; returns ((status N) (stdout "...") (stderr "..."))
```

### File I/O

```
(read "path")                ; read file to string
(read-lines "path")          ; read file to list of lines
(read-range "path" start end) ; read lines start..end (1-indexed, inclusive)
(write "path" "content")     ; write/overwrite file
(write-range "path" start end content) ; replace lines start..end with content
(append "path" "content")    ; append to file
(insert-at "path" line content) ; insert content before line N
(remove-range "path" start end) ; delete lines start..end
(exists "path")              ; bool
(file? "path")               ; true if regular file
(dir? "path")                ; true if directory
(file-size "path")           ; bytes as number
(mtime "path")               ; modification time as unix timestamp
(touch "path")               ; create or update timestamp
(rm "path")                  ; delete file or dir (alias: delete)
(mkdir "path")               ; recursive mkdir
(cp "src" "dst")             ; copy file or directory recursively (alias: copy)
(mv "src" "dst")             ; move/rename (alias: move)
(ls "path")                  ; list dir, returns sorted list of names (alias: list-dir)
(glob "pattern")             ; glob with * and **, returns list of paths
(cwd)                        ; current dir as string (alias: pwd)
(cd "path")                  ; change dir
(basename "path")            ; file name without directory
(dirname "path")             ; directory without file name
(ext "path")                 ; file extension without dot
(join-path a b...)           ; join path segments
(realpath "path")            ; resolve to canonical absolute path
```

Content for `write-range` and `insert-at` can be a string (split on newlines) or a list of strings.

### Environment

```
(getenv "NAME")              ; returns string or nil
(setenv "NAME" "val")        ; set env var
(env)                        ; all vars as ((key val) ...) list
```

### Strings

```
(str a b...)                 ; concatenate to string
(split "s" "delim")          ; returns list
(join list "delim")          ; list of strings -> string
(trim "s")                   ; strip whitespace
(contains "hay" "needle")    ; bool
(starts-with "s" "prefix")   ; bool
(ends-with "s" "suffix")     ; bool
(replace "s" "old" "new")    ; first occurrence
(upper "s") / (lower "s")   ; case
(substr "s" start len)       ; substring by byte index
(find "hay" "needle")        ; index or -1
(format "{} + {}" a b)       ; Python-style {} placeholders, also {0} {1} positional
```

### Regular Expressions

```
; Syntax
.         ; any char
\d / \D   ; digit / non-digit
\w / \W   ; word char (a-zA-Z0-9_) / non-word
\s / \S   ; whitespace / non-whitespace
[abc]     ; char class
[a-z]     ; range
[^abc]    ; negated class
* + ?     ; quantifiers
|         ; alternation
(...)     ; group
^ $       ; anchors

; Functions
(re-test pattern string)      ; bool: pattern matches anywhere?
(re-match pattern string)     ; list or nil: matches entire string? returns (match start end)
(re-find pattern string)      ; string or nil: first match
(re-find-all pattern string)  ; list of matched strings
(re-replace string pat repl)  ; replace first occurrence
(re-replace-all string pat repl) ; replace all occurrences
(re-split pattern string)     ; split by pattern, returns list
(re-scan pattern string)      ; list of (match start end) tuples
```

### Lists

```
(list a b...)                ; create list
(car list) / (head) / (first)   ; first element
(cdr list) / (tail) / (rest)    ; all but first
(cons elem list)             ; prepend
(len list-or-string)         ; length
(push list elem)             ; returns new list with elem appended
(nth list index)             ; element at index
(list?) (nil?) (empty?)      ; predicates
(last list)                  ; last element
(reverse list)               ; reversed copy
(sort list)                  ; sorted by string comparison
(flatten list)               ; deep flatten
```

Higher-order:
```
(map fn list)                ; returns new list
(filter fn list) / (select)
(reduce fn init list) / (fold)
(each fn list) / (for-each)  ; side effects, returns nil
(range end) / (range start end) / (range start end step)
(any fn list)                ; any true?
(all fn list)                ; all true?
(zip list...)                ; element-wise tuple
```

Object (assoc list) helpers:
```
(assoc alist key val)        ; add/update key (returns new list)
(dissoc alist key)           ; remove key
(keys alist)                 ; list of keys
(values alist)               ; list of values
(merge list...)              ; concatenate
```

### Arithmetic

```
(+ a b...)    ; add (also string concat)
(- a b...)    ; subtract (unary negate with 1 arg)
(* a b...)    ; multiply
(/ a b...)    ; divide
(% a b)       ; modulo  (also: mod)
(pow a b)     ; power
(sqrt a)      ; square root
(abs a)       ; absolute
(min a b...)  /  (max a b...)
(floor a)  (ceil a)  (round a)
(rand)        ; 0.0 to 1.0
(rand n)      ; random int 0..n-1
(inc a)       ; a + 1
(dec a)       ; a - 1
```

### Comparison

```
(= a b)  / (== a b)    ; equal
(!= a b)                ; not equal
(< a b)  (> a b)  (<= a b)  (>= a b)
```

Works on numbers, strings, bools, nil. Lists compared structurally.

### Logic

```
(not x)     ; boolean negation
(and ...)   ; short-circuit, returns first falsy or last value
(or  ...)   ; short-circuit, returns first truthy or last value
```

### Type

```
(type x)        ; returns "number"/"string"/"bool"/"nil"/"list"/"function"/"symbol"/"builtin"
(int x)         ; to integer (truncates)
(float x)       ; to float
(number? x)  (string? x)  (list? x)  (nil? x)  (bool? x)
```

### IO

```
(print a...)            ; stdout, no newline, args joined by space
(println a...)          ; stdout, with newline
(eprint a...)           ; stderr
(eprintln a...)         ; stderr, newline
(input "prompt? ")      ; read line from stdin
```

### HTTP (via curl)

```
(http-get "url")                              ; returns response body string
(http-post "url" "body")                      ; POST with body
(http-put "url" "body")                       ; PUT
(http-delete "url")                           ; DELETE
(http "METHOD" "url" ?body? ?headers-list?)   ; full control
```

### JSON

JSON <-> alisp mapping:
- `null` <-> `nil`
- `true`/`false` <-> `true`/`false`
- number <-> number
- string <-> string
- array `[1,2,3]` <-> list `(1 2 3)`
- object `{"a":1}` <-> `(("a" 1))` (list of key-value pairs)

```
(json-parse "json-string")        ; parse JSON to alisp (also: json)
(json-stringify expr)             ; to pretty JSON
(json-stringify expr "compact")   ; to compact JSON (also: json-str)
(json-get obj key)                ; get by string key or int index (also: jget)
(json-set obj key val)            ; returns new object (also: jset)
(json-keys obj)                   ; list of string keys
```

### Misc

```
(sleep N)           ; seconds (float ok)
(time)              ; elapsed seconds since start
(timestamp)         ; unix epoch seconds
(exit) / (exit N)   ; quit interpreter
```

## Patterns

```
; Pipe chain
(def result
  (-> (exec "ps aux")
      (split "\n")
      (len)))

; Safe execution
(try (exec "risky") (catch e (println "failed:" e)))

; Accumulate in loop
(def acc (list))
(dotimes (i 5)
  (set! acc (push acc (* i i))))

; Build JSON response
(def obj (json-parse "{}"))
(def obj (json-set obj "status" "ok"))
(def obj (json-set obj "count" (len items)))
(println (json-stringify obj "compact"))

; Parse structured output with regex
(def lines (exec "ps aux"))
(def pids (re-find-all "\\d+" lines))

; Validate input
(if (re-test "^[a-z]+@[a-z]+\\.[a-z]+$" email)
  (println "valid email")
  (println "invalid email"))
```
