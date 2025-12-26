# Monad Ownership System

*An ownership and borrowing system for ML-style functional programming*

---

## Table of Contents

1. [Introduction](#introduction)
2. [Design Philosophy](#design-philosophy)
3. [Core Ownership Rules](#core-ownership-rules)
4. [Reference System](#reference-system)
5. [Borrowing System](#borrowing-system)
6. [Type System Integration](#type-system-integration)
7. [Data Structures](#data-structures)
8. [Memory Model and Allocation](#memory-model-and-allocation)
9. [Closure Capture Rules](#closure-capture-rules)
10. [Module System Integration](#module-system-integration)
11. [Concurrency and Thread Safety](#concurrency-and-thread-safety)
12. [Examples](#examples)
13. [Error Reference](#error-reference)

---

## 1. Introduction

Monad is a functional programming language that combines SML's elegant syntax with compile-time memory safety through ownership. Unlike traditional ML dialects that rely on garbage collection, Monad uses **linear ownership** to provide deterministic memory management without runtime overhead.

### Key Goals

- Preserve ML's immutability-by-default semantics
- Maintain pattern matching, algebraic data types, and type inference
- Provide memory safety without garbage collection
- Keep syntax clean and familiar to ML programmers
- Support safe multi-threaded programming
- Minimize explicit annotations

### What Makes Monad Different

- **No GC overhead**: Deterministic destruction when owners go out of scope
- **Linear ownership**: Each value has exactly one owner
- **Compile-time safety**: Prevent use-after-free and data races statically
- **ML aesthetics**: Feels like SML, not Rust
- **Systems-level**: Suitable for performance-critical code

---

## 2. Design Philosophy

### 2.1 Immutable by Default

All bindings are immutable unless explicitly made mutable with references:

```sml
val x = 10              (* immutable binding *)
val y = &mut 10         (* mutable reference *)
```

### 2.2 Explicit Over Implicit

- Mutation requires `&mut` and `:=`
- Copying heap values requires explicit `clone`
- Reference types are explicit: `&T` vs `&mut T`
- No hidden allocations or conversions

### 2.3 Ownership Without Lifetime Annotations

The compiler infers ownership and borrows. No lifetime parameters in user code:

```sml
fun length lst =        (* compiler infers: borrows lst *)
  case lst of
    [] => 0
  | _ :: xs => 1 + length xs
```

### 2.4 Functional First

Ownership serves functional programming, not the other way around. Pattern matching, higher-order functions, and algebraic data types work naturally.

### 2.5 Minimal Data Structures

Monad provides exactly five core data structures:
- **string**: immutable text
- **array**: mutable, fixed-size
- **list**: immutable, persistent linked list
- **stack/queue**: mutable, imperative
- **map/hashtable**: both immutable and mutable variants

This simplicity makes the language easy to learn while covering all common use cases.

---

## 3. Core Ownership Rules

### 3.1 The Ownership Principle

**Every value has exactly one owner at any point in time.**

When the owner goes out of scope, the value is destroyed (freed if heap-allocated).

### 3.2 Move Semantics

Assignment and parameter passing **move** ownership for non-copyable types:

```sml
val x = [1, 2, 3]           (* x owns the list *)
val y = x                   (* ownership moves to y *)
(* print x *)               (* ERROR: x no longer owns a value *)
```

### 3.3 Copy Types

Small, trivial types are automatically **copyable**:

**Copy types:**
- `int`, `bool`, `char`, `unit`, `real`
- Tuples of copy types: `int * bool`
- Function pointers (not closures capturing non-copy values)

```sml
val x = 42
val y = x                   (* x is copied, both valid *)
val z = x + y               (* OK: x still valid *)
```

### 3.4 Move Types

Heap-allocated and complex types are **move types**:

**Move types:**
- `string`
- `'a list`
- `'a array`
- `'a stack`, `'a queue`
- `('k, 'v) map`, `('k, 'v) hashtable`
- Algebraic data types with non-copy fields
- Closures capturing moved values
- References: `&T` and `&mut T`

```sml
datatype tree = Leaf | Node of int * tree * tree

val t = Node (5, Leaf, Leaf)
val t2 = t                  (* t moved, now invalid *)
```

### 3.5 Partial Moves Are Forbidden

You cannot move individual fields out of a structure:

```sml
val pair = ([1, 2], [3, 4])
val x = #1 pair             (* ERROR: would leave pair partially moved *)
```

**Solution:** Destructure the entire value:

```sml
val (x, y) = pair           (* OK: entire pair consumed, x and y own parts *)
```

---

## 4. Reference System

### 4.1 Reference Types

Monad uses **explicit reference syntax** to distinguish mutable from immutable:

```sml
&T              (* immutable (shared) reference *)
&mut T          (* mutable (exclusive) reference *)
```

### 4.2 Creating References

```sml
(* Immutable reference *)
val x = 42
val r = &x              (* immutable reference to x *)

(* Mutable reference *)
val m = &mut 10         (* mutable reference with value 10 *)
```

### 4.3 Dereferencing

Use `*` to dereference both immutable and mutable references:

```sml
val r = &42
val x = *r              (* x = 42 *)

val m = &mut 10
val y = *m              (* y = 10 *)
```

### 4.4 Mutation

Only `&mut T` references can be mutated using `:=`:

```sml
val m = &mut 10
m := 20                 (* OK: m is mutable *)

val r = &10
r := 20                 (* ERROR: cannot mutate &T, need &mut T *)
```

### 4.5 Reference Semantics

References themselves are values subject to ownership:

```sml
val r1 = &mut 10
val r2 = r1             (* r1 moved to r2 *)
(* *r1 *)              (* ERROR: r1 was moved *)

val r3 = &42
val r4 = r3             (* r3 copied: references are lightweight *)
val x = *r3             (* OK: immutable refs can be copied *)
```

### 4.6 Type Signatures with References

```sml
(* Function signatures make mutability explicit *)
fun read_value (r: &int) : int = *r
fun increment (r: &mut int) : unit = r := *r + 1

(* Data structures *)
type person = {
  name: string,
  age: &mut int         (* mutable field via reference *)
}

datatype counter = Counter of {
  label: string,
  count: &mut int
}
```

---

## 5. Borrowing System

### 5.1 Motivation

Without borrowing, every function that inspects a value would consume it. Borrowing allows non-destructive access.

### 5.2 Shared Borrows (Read-Only)

**Multiple readers, no writers.**

Functions that only inspect their arguments automatically borrow:

```sml
fun isEmpty lst =
  case lst of
    [] => true
  | _ => false

val myList = [1, 2, 3]
val empty = isEmpty myList  (* myList borrowed, still valid *)
val len = length myList     (* OK: can borrow again *)
```

### 5.3 Exclusive Borrows (Mutable)

**One writer, no other access.**

Mutation requires an exclusive borrow via `&mut`:

```sml
val counter = &mut 0

fun increment (r: &mut int) : unit =
  r := *r + 1

increment (&mut counter)    (* exclusive borrow *)
increment (&mut counter)    (* OK: sequential borrows *)
```

### 5.4 Borrow Inference Rules

The compiler infers borrows based on usage:

| **Usage Pattern** | **Borrow Type** |
|-------------------|-----------------|
| Read-only inspection | Shared |
| Mutation via `:=` | Exclusive |
| Destructuring or return | Move |
| Copy type access | Copy (no borrow) |

```sml
fun readFirst lst =
  case lst of
    [] => NONE
  | x :: _ => SOME x        (* borrows lst, copies x (if copy type) *)
```

### 5.5 Borrow Lifetime Rules

A borrow's lifetime extends from creation to last use:

```sml
val data = [1, 2, 3]
val len = length data       (* borrow starts and ends here *)
val data2 = data            (* OK: borrow ended, can move *)
```

**Borrows cannot outlive the owned value:**

```sml
fun makeBorrow () =
  let val temp = [1, 2, 3]
  in temp                   (* OK: moves temp (heap allocated) *)
  end

(* But this fails: *)
fun makeBadClosure () =
  let val temp = [1, 2, 3]
  in fn () => length temp   (* ERROR: closure borrows temp, which dies *)
  end
```

### 5.6 Borrowing with `&` Syntax

Explicit borrowing at call sites:

```sml
val arr = array 10 0
set (&mut arr) 5 42         (* explicit exclusive borrow *)
val x = get (&arr) 5        (* explicit shared borrow *)
```

---

## 6. Type System Integration

### 6.1 Type Syntax

Monad uses standard ML type syntax with reference annotations:

```sml
datatype 'a option = NONE | SOME of 'a
datatype 'a list = [] | :: of 'a * 'a list

type person = { name: string, age: int }

(* Reference types *)
type counter = &mut int
type readonly = &string
```

### 6.2 Ownership in Types

Ownership is tracked by the compiler but not written in basic types:

```sml
(* User writes: *)
fun identity x = x

(* Compiler understands: *)
(* - If x is copy type: copies x *)
(* - If x is move type: moves x *)
```

### 6.3 Parametric Polymorphism

Type parameters respect ownership:

```sml
fun identity (x : 'a) : 'a = x

val n = identity 42         (* copies 42 *)
val lst = identity [1,2,3]  (* moves list *)
```

### 6.4 Algebraic Data Types

ADTs follow ownership rules:

```sml
datatype 'a tree = 
    Leaf 
  | Node of 'a * 'a tree * 'a tree

val t = Node (5, Leaf, Leaf)
val t2 = t                  (* t moved *)

fun height tree =
  case tree of
    Leaf => 0
  | Node (_, left, right) => 
      1 + max (height left) (height right)
      (* left and right borrowed by recursive calls *)
```

---

## 7. Data Structures

### 7.1 String (Immutable)

```sml
type string          (* immutable *)

val concat : string -> string -> string
val substring : &string -> int -> int -> string
val length : &string -> int
val compare : &string -> &string -> int

(* String building *)
type string_buffer   (* mutable builder *)
val buffer : unit -> string_buffer
val append : &mut string_buffer -> string -> unit
val to_string : string_buffer -> string
```

**Characteristics:**
- Immutable by default
- Safe to share across threads
- Efficient substring sharing
- Use `string_buffer` for construction

**Default Allocation: HEAP**
- String literals: heap-allocated
- String operations create heap allocations
- Small string optimization may apply (< 24 bytes on stack)
- Rationale: Strings are dynamically sized and often escape

**Example:**
```sml
val s1 = "hello"               (* HEAP: string literal *)
val s2 = concat s1 " world"    (* HEAP: result string, s1 unchanged *)

(* Building strings efficiently *)
val buf = buffer ()            (* HEAP: dynamic buffer *)
append (&mut buf) "hello"
append (&mut buf) " world"
val result = to_string buf     (* HEAP: final string *)
```

### 7.2 Array (Mutable)

```sml
type 'a array        (* mutable, fixed-size *)

val array : int -> 'a -> 'a array
val get : &'a array -> int -> 'a
val set : &mut 'a array -> int -> 'a -> unit
val length : &'a array -> int
val copy : &'a array -> 'a array
```

**Characteristics:**
- Mutable, random access
- Fixed size after creation
- O(1) get/set operations
- Use for performance-critical code

**Default Allocation: DEPENDS ON SIZE AND ESCAPE**
- Small arrays (< 4KB) that don't escape: STACK
- Large arrays (≥ 4KB): HEAP
- Arrays that escape scope: HEAP
- Rationale: Small local arrays benefit from stack allocation

**Allocation Examples:**
```sml
(* STACK: small, local *)
fun quick_lookup () : int =
  let val lookup_table = array 10 0    (* STACK: 40 bytes, doesn't escape *)
  in get (&lookup_table) 5
  end

(* HEAP: returned *)
fun make_array () : int array =
  array 10 0                           (* HEAP: escapes function *)

(* HEAP: large *)
fun process_large () : int =
  let val big = array 100000 0         (* HEAP: 400KB exceeds threshold *)
  in get (&big) 0
  end

(* HEAP: captured *)
fun make_stateful () : (int -> unit) =
  let val state = array 5 0            (* HEAP: captured by closure *)
  in fn i => set (&mut state) i 42
  end
```

**Example:**
```sml
val arr = array 10 0
set (&mut arr) 5 42         (* in-place mutation *)
val x = get (&arr) 5        (* read: borrows immutably *)

(* Ownership prevents data races *)
val arr2 = arr              (* arr moved to arr2 *)
spawn (fn () => set (&mut arr2) 0 99)   (* OK: exclusive ownership *)
```

### 7.3 List (Immutable)

```sml
datatype 'a list = [] | :: of 'a * 'a list

val append : 'a list -> 'a list -> 'a list
val map : ('a -> 'b) -> &'a list -> 'b list
val filter : ('a -> bool) -> &'a list -> 'a list
val fold : ('a * 'b -> 'b) -> 'b -> &'a list -> 'b
val length : &'a list -> int
val rev : 'a list -> 'a list
```

**Characteristics:**
- Immutable, persistent
- Structural sharing
- O(1) cons operation
- O(n) append, lookup

**Default Allocation: HEAP (Always)**
- Lists are recursive structures with unbounded size
- Each cons cell is heap-allocated
- Empty list `[]` may be a special singleton (no allocation)
- Rationale: Cannot determine size at compile time

**Allocation Examples:**
```sml
(* HEAP: all list nodes *)
val xs = [1, 2, 3]                     (* HEAP: three cons cells *)
val ys = 0 :: xs                       (* HEAP: one new cons cell, shares xs *)
val empty = []                         (* No allocation: singleton *)

(* Local list still heap-allocated *)
fun local_list () : int =
  let val temp = [1, 2, 3]             (* HEAP: recursive structure *)
  in length (&temp)
  end                                  (* temp freed here *)
```

**Example:**
```sml
val xs = [1, 2, 3]
val ys = 0 :: xs            (* shares xs, no copy *)
val zs = map (fn x => x * 2) (&xs)   (* xs still valid *)

(* Safe sharing across threads *)
spawn (fn () => length (&xs))
spawn (fn () => fold op+ 0 (&xs))
```

### 7.4 Stack and Queue (Mutable)

```sml
(* Stack - LIFO *)
type 'a stack

val stack : unit -> 'a stack
val push : &mut 'a stack -> 'a -> unit
val pop : &mut 'a stack -> 'a option
val peek : &'a stack -> 'a option
val is_empty : &'a stack -> bool

(* Queue - FIFO *)
type 'a queue

val queue : unit -> 'a queue
val enqueue : &mut 'a queue -> 'a -> unit
val dequeue : &mut 'a queue -> 'a option
val peek : &'a queue -> 'a option
val is_empty : &'a queue -> bool
```

**Characteristics:**
- Mutable, imperative data structures
- Efficient push/pop and enqueue/dequeue
- Used in algorithms: DFS, BFS, parsing

**Default Allocation: HEAP (Always)**
- Stacks and queues are dynamically sized
- Internal storage grows as needed
- Even small stacks/queues use heap
- Rationale: Size varies at runtime, typically escape scope

**Allocation Examples:**
```sml
(* HEAP: always *)
fun dfs (graph: graph) (start: node) : unit =
  let val s = stack ()                 (* HEAP: dynamic structure *)
  in
    push (&mut s) start;               (* may grow dynamically *)
    (* ... *)
  end

(* HEAP: even if returned *)
fun make_work_queue () : string queue =
  queue ()                             (* HEAP: escapes function *)

(* HEAP: captured *)
fun make_processor () : (int -> unit) =
  let val q = queue ()                 (* HEAP: captured by closure *)
  in fn item => enqueue (&mut q) item
  end
```

**Example:**
```sml
(* DFS with stack *)
fun dfs (graph: graph) (start: node) : unit =
  let val s = stack ()
      val visited = hashset ()
  in
    push (&mut s) start;
    while not (is_empty (&s)) do
      case pop (&mut s) of
        SOME node =>
          if not (contains (&visited) node) then
            (process node;
             insert (&mut visited) node;
             List.iter (fn n => push (&mut s) n) (neighbors graph node))
          else ()
      | NONE => ()
  end
```

### 7.5 Map (Immutable) and HashTable (Mutable)

```sml
(* Immutable map - persistent balanced tree *)
type ('k, 'v) map

val empty : ('k, 'v) map
val insert : ('k, 'v) map -> 'k -> 'v -> ('k, 'v) map
val lookup : &('k, 'v) map -> 'k -> 'v option
val remove : ('k, 'v) map -> 'k -> ('k, 'v) map
val keys : &('k, 'v) map -> 'k list
val values : &('k, 'v) map -> 'v list

(* Mutable hashtable - hash map *)
type ('k, 'v) hashtable

val hashtable : int -> ('k, 'v) hashtable
val insert : &mut ('k, 'v) hashtable -> 'k -> 'v -> unit
val lookup : &('k, 'v) hashtable -> 'k -> 'v option
val remove : &mut ('k, 'v) hashtable -> 'k -> unit
val keys : &('k, 'v) hashtable -> 'k list
val values : &('k, 'v) hashtable -> 'v list
```

**When to use Map:**
- Functional algorithms
- Small to medium size
- Need historical versions
- Persistence matters

**When to use HashTable:**
- Large data sets
- Performance critical
- Caching, memoization
- Frequency counting

**Default Allocation:**

**Map (Immutable): HEAP (Always)**
- Implemented as balanced tree (AVL/Red-Black)
- Each node is heap-allocated
- Empty map may be singleton (no allocation)
- Rationale: Recursive tree structure, supports structural sharing

**HashTable (Mutable): HEAP (Always)**
- Implemented with array of buckets
- Dynamically resizes as load factor increases
- Internal arrays always heap-allocated
- Rationale: Variable size, typically escapes or is captured

**Allocation Examples:**
```sml
(* Map - HEAP: tree nodes *)
val m1 = empty                         (* No allocation: singleton *)
val m2 = insert m1 "key" 42            (* HEAP: new tree node *)
val m3 = insert m2 "key2" 43           (* HEAP: shares structure with m2 *)

(* HashTable - HEAP: bucket array *)
val table = hashtable 100              (* HEAP: array + metadata *)
insert (&mut table) "key" 42           (* may trigger resize -> HEAP *)

(* Local map still heap-allocated *)
fun count_words (words: string list) : (string, int) map =
  let val m = empty                    (* singleton *)
  in fold (fn (w, acc) => 
       insert acc w 1) m words         (* HEAP: builds tree nodes *)
  end
```

**Example:**
```sml
(* Map: functional style *)
fun eval (expr: expr) (env: (string, int) map) : int =
  case expr of
    Var x => Option.get (lookup (&env) x)
  | Let (x, e1, e2) => 
      let val v = eval e1 env
      in eval e2 (insert env x v)    (* new env, old preserved *)
      end

(* HashTable: imperative style *)
fun word_frequency (words: string list) : (string, int) hashtable =
  let val table = hashtable 10000
  in
    List.iter (fn word =>
      let val count = Option.getOrElse (lookup (&table) word) 0
      in insert (&mut table) word (count + 1)
      end
    ) words;
    table
  end
```

---

## 8. Memory Model and Allocation

### 8.1 Stack vs Heap Allocation

Values start on the **stack** by default. The compiler promotes to **heap** based on escape analysis:

**Principle:** Stack when possible, heap when necessary.

### 8.2 Allocation Rules

#### Rule 1: Non-Escaping Values → Stack

```sml
fun compute () : int =
  let val x = [1, 2, 3]        (* STACK: used only locally *)
      val y = array 10 0       (* STACK: doesn't escape *)
  in length (&x) + get (&y) 0
  end
```

#### Rule 2: Returned Values → Heap

```sml
fun make_list () : int list =
  [1, 2, 3]                    (* HEAP: returned, outlives function *)

fun make_array () : int array =
  array 10 0                   (* HEAP: returned *)
```

#### Rule 3: Captured by Escaping Closure → Heap

```sml
fun make_counter () : (unit -> int) =
  let val count = &mut 0       (* HEAP: captured by returned closure *)
  in fn () => (count := *count + 1; *count)
  end

fun local_closure () : int =
  let val x = 10
      val f = fn () => x + 1   (* STACK: closure doesn't escape *)
  in f ()
  end
```

#### Rule 4: Stored in Heap Structure → Heap

```sml
datatype tree = Leaf | Node of int * tree * tree

fun build_tree () : tree =
  let val left = Node (1, Leaf, Leaf)    (* HEAP: stored in parent *)
      val right = Node (3, Leaf, Leaf)   (* HEAP: stored in parent *)
  in Node (2, left, right)               (* HEAP: returned *)
  end
```

#### Rule 5: Large Values → Heap

```sml
fun process_large_data () : int =
  let val big_array = array 100000 0    (* HEAP: exceeds stack threshold *)
  in get (&big_array) 0
  end
```

**Size threshold:** ~4KB recommended

#### Rule 6: Recursive Structures → Always Heap

```sml
datatype 'a list = [] | :: of 'a * 'a list

val xs = 1 :: 2 :: 3 :: []    (* HEAP: recursive structure *)

datatype tree = Leaf | Node of int * tree * tree
val t = Node (5, Leaf, Leaf)  (* HEAP: recursive structure *)
```

#### Rule 7: Structs with Heap Fields → Heap

```sml
type person = {
  name: string,                (* heap-allocated string *)
  age: int
}

fun make_person () : person =
  { name = "Alice", age = 30 } (* HEAP: contains heap field *)
```

#### Rule 8: Small Structs (All-Copy) → Stack

```sml
type point = { x: int, y: int }

fun distance () : real =
  let val p1 = { x = 0, y = 0 }        (* STACK: small, all copy fields *)
      val p2 = { x = 3, y = 4 }        (* STACK *)
  in sqrt (real ((#x p2 - #x p1)^2 + (#y p2 - #y p1)^2))
  end
```

### 8.3 Escape Analysis

A value **escapes** if it:
- Is returned from a function
- Is stored in a heap-allocated structure
- Is captured by an escaping closure
- Outlives its creating scope

### 8.4 Destruction Rules

When an owner goes out of scope:
- **Stack values**: Automatically destroyed, stack unwound
- **Heap values**: Freed (no GC needed)
- **Moved values**: Skip destruction (new owner handles it)

```sml
fun example () =
  let val x = [1, 2, 3]       (* heap allocated *)
      val y = x               (* x moved to y *)
  in length (&y)
  end                         (* y destroyed, list freed; x skipped *)
```

### 8.5 No Garbage Collection

Monad does not require a garbage collector:
- Destruction is deterministic
- Deallocation happens at scope exit
- No runtime overhead for tracing or collection

### 8.6 Allocation Summary

| Scenario | Location | Reason |
|----------|----------|--------|
| Local primitive | Stack | Copy type, doesn't escape |
| Local small struct (all-copy) | Stack | Small, all fields copyable |
| Local array (small) | Stack | Doesn't escape, under threshold |
| Local array (large) | Heap | Exceeds size threshold (~4KB) |
| Returned value | Heap | Escapes function scope |
| Struct with heap fields | Heap | Contains non-copy fields |
| Recursive type | Heap | Unbounded size |
| Captured by escaping closure | Heap | Outlives creating scope |
| Stored in heap structure | Heap | Parent is heap-allocated |

---

## 9. Closure Capture Rules

### 9.1 Capture by Move

Closures that use non-copyable values **move** them:

```sml
val data = [1, 2, 3]
val f = fn () => length (&data)    (* data moved into closure *)
(* length (&data) *)                (* ERROR: data was moved *)
```

### 9.2 Capture by Copy

Copyable values are copied into closures:

```sml
val x = 42
val f = fn () => x + 1          (* x copied *)
val y = x + 10                  (* OK: x still valid *)
```

### 9.3 Capture by Reference

Closures can capture references:

```sml
val count = &mut 0
val increment = fn () => count := *count + 1
increment ()                    (* OK: borrows count *)
```

### 9.4 Escaping Closures

Closures that escape must own or copy captured values:

```sml
fun makeCounter () : (unit -> int) =
  let val count = &mut 0        (* HEAP: captured by escaping closure *)
  in fn () => (count := *count + 1; *count)
  end

(* count is heap-allocated and owned by returned closure *)
```

---

## 10. Module System Integration

### 10.1 Module Signatures

Signatures describe ownership through function types:

```sml
signature STACK = sig
  type 'a t
  val empty : 'a t
  val push : &mut 'a t -> 'a -> unit
  val pop : &mut 'a t -> 'a option
  val peek : &'a t -> 'a option
end
```

### 10.2 Implementation Example

```sml
structure Stack :> STACK = struct
  type 'a t = 'a list
  
  val empty = []
  
  fun push s x = s := x :: *s
  
  fun pop s = 
    case *s of
      [] => NONE
    | x :: xs => (s := xs; SOME x)
  
  fun peek s =
    case *s of
      [] => NONE
    | x :: _ => SOME x
end
```

### 10.3 Functors

Functors preserve ownership semantics:

```sml
functor MakeSet (Ord: ORDERED) :> SET = struct
  type elem = Ord.t
  type set = elem list
  
  fun insert (s: &mut set) (x: elem) : unit =
    if not (member s x) then
      s := x :: *s
    else ()
  
  fun member (s: &set) (x: elem) : bool =
    List.exists (fn y => Ord.eq (x, y)) (*s)
end
```

---

## 11. Concurrency and Thread Safety

### 11.1 Thread Safety Principles

**Rule 1: Immutable data is safe to share**

```sml
val shared_config = load_config ()
spawn (fn () => use_config (&shared_config))    (* OK: immutable *)
spawn (fn () => use_config (&shared_config))    (* OK: multiple readers *)
```

**Rule 2: Mutable data requires exclusive ownership**

```sml
val mut_data = array 100 0
spawn (fn () => set (&mut mut_data) 0 42)       (* ERROR: ownership violation *)
(* Cannot share &mut across threads *)
```

**Rule 3: Transfer ownership to share mutable data**

```sml
val data = array 100 0
spawn (fn () => 
  let val local_data = data    (* data moved to thread *)
  in set (&mut local_data) 0 42
  end
)
(* data invalid in parent thread *)
```

### 11.2 Send and Sync Traits (Implicit)

Types are implicitly:
- **Send**: Can transfer ownership across threads
- **Sync**: Can share immutable references across threads

| Type | Send | Sync | Notes |
|------|------|------|-------|
| `int`, `bool` | ✓ | ✓ | Copy types |
| `string` | ✓ | ✓ | Immutable |
| `'a list` | ✓ | ✓ | Immutable |
| `'a array` | ✓ | ✗ | Mutable, needs exclusive |
| `&T` | ✗ | ✓ | Can share reference |
| `&mut T` | ✗ | ✗ | Exclusive access only |

### 11.3 Message Passing

```sml
type 'a channel

val channel : unit -> 'a channel
val send : &'a channel -> 'a -> unit
val receive : &'a channel -> 'a

(* Safe message passing *)
val ch = channel ()
val data = [1, 2, 3]

spawn (fn () => 
  let val msg = receive (&ch)
  in process msg
  end
)

send (&ch) data              (* data moved into channel *)
(* data invalid here *)
```

### 11.4 Atomic Operations

For fine-grained concurrency:

```sml
type atomic_int

val atomic : int -> atomic_int
val load : &atomic_int -> int
val store : &atomic_int -> int -> unit
val fetch_add : &atomic_int -> int -> int
val compare_exchange : &atomic_int -> int -> int -> bool

(* Thread-safe counter *)
val counter = atomic 0

spawn (fn () => fetch_add (&counter) 1)
spawn (fn () => fetch_add (&counter) 1)
(* Safe: atomic operations *)
```

---

## 12. Examples

### 12.1 Functional List Processing

```sml
(* Map borrows the list *)
fun map f [] = []
  | map f (x :: xs) = f x :: map f xs

val nums = [1, 2, 3]
val doubled = map (fn x => x * 2) (&nums)    (* nums borrowed *)
val tripled = map (fn x => x * 3) (&nums)    (* OK: nums still valid *)
```

### 12.2 Binary Tree

```sml
datatype 'a tree = Leaf | Node of 'a * 'a tree * 'a tree

fun height tree =
  case tree of
    Leaf => 0
  | Node (_, left, right) =>
      1 + max (height (&left)) (height (&right))

val t = Node (5, Node (3, Leaf, Leaf), Leaf)
val h = height (&t)                          (* t borrowed *)
val h2 = height (&t)                         (* OK *)
```

### 12.3 Mutable Counter

```sml
type counter = { label: string, count: &mut int }

fun make_counter (label: string) : counter =
  { label = label, count = &mut 0 }

fun increment (c: &counter) : unit =
  #count c := *(#count c) + 1

fun reset (c: &counter) : unit =
  #count c := 0

fun get (c: &counter) : int =
  *(#count c)

val c = make_counter "clicks"
increment (&c)                               (* borrows c *)
increment (&c)                               (* OK *)
val n = get (&c)                             (* n = 2 *)
```

### 12.4 Higher-Order Functions

```sml
fun compose f g x = f (g x)

val add1 = fn x => x + 1
val double = fn x => x * 2
val add1ThenDouble = compose double add1

val result = add1ThenDouble 5                (* 12 *)
```

### 12.5 Hash Table Caching

```sml
type 'a cache = (string, 'a) hashtable

fun make_cache () : 'a cache =
  hashtable 100

fun get_or_compute (cache: &'a cache) (key: string) (compute: unit -> 'a) : 'a =
  case lookup cache key of
    SOME value => value
  | NONE =>
      let val value = compute ()
      in
        insert (&mut cache) key value;
        value
      end

val cache = make_cache ()
val result = get_or_compute (&cache) "expensive" 
               (fn () => expensive_computation ())
```

### 12.6 Graph Traversal

```sml
type graph = (node, node list) hashtable

fun bfs (g: &graph) (start: node) : unit =
  let val q = queue ()
      val visited = hashset ()
  in
    enqueue (&mut q) start;
    while not (is_empty (&q)) do
      case dequeue (&mut q) of
        SOME node =>
          if not (contains (&visited) node) then
            (process node;
             insert (&mut visited) node;
             case lookup g node of
               SOME neighbors => 
                 List.iter (fn n => enqueue (&mut q) n) neighbors
             | NONE => ())
          else ()
      | NONE => ()
  end
```

---

## 13. Error Reference

### 13.1 Use After Move

**Code:**
```sml
val x = [1, 2, 3]
val y = x
length (&x)
```

**Error:**
```
Error: value 'x' was moved and is no longer accessible
  moved here: val y = x
  used here: length (&x)
```

**Fix:** Use the new owner or clone before moving:
```sml
val x = [1, 2, 3]
val y = clone (&x)
length (&x)                (* OK *)
```

### 13.2 Cannot Mutate Immutable Reference

**Code:**
```sml
val r = &10
r := 20
```

**Error:**
```
Error: cannot mutate immutable reference
  type: &int (immutable reference)
  attempted mutation: r := 20
  
Help: use &mut int for mutable references
```

**Fix:**
```sml
val r = &mut 10
r := 20                    (* OK *)
```

### 13.3 Overlapping Borrows

**Code:**
```sml
val arr = array 10 0
val r1 = &mut arr
val r2 = &mut arr          (* ERROR: second exclusive borrow *)
set r1 0 42
```

**Error:**
```
Error: cannot borrow 'arr' as mutable more than once
  first mutable borrow: val r1 = &mut arr
  second mutable borrow: val r2 = &mut arr
```

**Fix:** Ensure borrows don't overlap:
```sml
val arr = array 10 0
set (&mut arr) 0 42        (* borrow ends immediately *)
set (&mut arr) 1 43        (* OK: sequential borrows *)
```

### 13.4 Borrow Outlives Owner

**Code:**
```sml
fun get_reference () : &int =
  let val temp = 42
  in &temp                 (* ERROR: temp dies at end of let *)
  end
```

**Error:**
```
Error: 'temp' does not live long enough
  'temp' is dropped here: end of 'let' expression
  but reference is returned here: &temp
```

**Fix:** Return the value, not a reference:
```sml
fun get_value () : int =
  let val temp = 42
  in temp                  (* OK: copies value *)
  end
```

### 13.5 Closure Capturing Moved Value

**Code:**
```sml
val data = [1, 2, 3]
val f = fn () => length (&data)
length (&data)
```

**Error:**
```
Error: value 'data' was moved and is no longer accessible
  moved here: val f = fn () => ... (captured by closure)
  used here: length (&data)
```

**Fix:** Clone the data or redesign:
```sml
val data = [1, 2, 3]
val data_copy = clone (&data)
val f = fn () => length (&data_copy)
length (&data)             (* OK *)
```

### 13.6 Partial Move

**Code:**
```sml
val pair = ([1, 2], [3, 4])
val x = #1 pair
```

**Error:**
```
Error: cannot move out of tuple
  moving field would leave 'pair' partially moved
```

**Fix:** Destructure the entire value:
```sml
val (x, y) = pair          (* OK: entire pair consumed *)
```

### 13.7 Thread Safety Violation

**Code:**
```sml
val arr = array 100 0
spawn (fn () => set (&mut arr) 0 42)
spawn (fn () => set (&mut arr) 1 43)
```

**Error:**
```
Error: cannot share mutable array across threads
  'arr' is accessed mutably in multiple threads
  type: int array (not thread-safe for shared mutation)
  
Help: transfer ownership to one thread, or use atomic operations
```

**Fix:** Transfer ownership or use message passing:
```sml
val arr = array 100 0
val ch = channel ()
send (&ch) arr             (* arr moved into channel *)

spawn (fn () => 
  let val local_arr = receive (&ch)
  in set (&mut local_arr) 0 42
  end
)
```

---

## Appendices

### A. Grammar Summary

```
(* Core syntax - ML-style *)
expr ::= 
  | val id = expr
  | fun id params = expr
  | fn params => expr
  | case expr of patterns
  | expr expr                    (* application *)
  | id
  | literal
  | ( expr )
  | [ expr, ... ]                (* list *)
  | expr :: expr                 (* cons *)
  | &expr                        (* immutable reference *)
  | &mut expr                    (* mutable reference *)
  | *expr                        (* dereference *)
  | expr := expr                 (* assignment *)
  | # label expr                 (* field access *)
  | { label = expr, ... }        (* record *)
  | datatype id = constructors
  | clone expr                   (* explicit deep copy *)
  | spawn expr                   (* spawn thread *)
  
(* Boolean operators *)
  | expr andalso expr            (* short-circuit AND *)
  | expr orelse expr             (* short-circuit OR *)
  | not expr                     (* logical NOT *)
```

### B. Standard Library Summary

**Core Operations:**
- `clone : &'a -> 'a` (deep copy for move types)
- `identity : 'a -> 'a` (moves non-copy types)

**String:**
- `concat : string -> string -> string`
- `substring : &string -> int -> int -> string`
- `length : &string -> int`

**Array:**
- `array : int -> 'a -> 'a array`
- `get : &'a array -> int -> 'a`
- `set : &mut 'a array -> int -> 'a -> unit`

**List:**
- `length : &'a list -> int`
- `map : ('a -> 'b) -> &'a list -> 'b list`
- `filter : ('a -> bool) -> &'a list -> 'a list`
- `fold : ('a * 'b -> 'b) -> 'b -> &'a list -> 'b`

**Stack/Queue:**
- `push : &mut 'a stack -> 'a -> unit`
- `pop : &mut 'a stack -> 'a option`
- `enqueue : &mut 'a queue -> 'a -> unit`
- `dequeue : &mut 'a queue -> 'a option`

**Map/HashTable:**
- `insert : ('k, 'v) map -> 'k -> 'v -> ('k, 'v) map`
- `insert : &mut ('k, 'v) hashtable -> 'k -> 'v -> unit`
- `lookup : &('k, 'v) map -> 'k -> 'v option`

**Concurrency:**
- `spawn : (unit -> unit) -> unit`
- `channel : unit -> 'a channel`
- `send : &'a channel -> 'a -> unit`
- `receive : &'a channel -> 'a`

### C. Comparison with Other Languages

| Feature | Monad | Rust | OCaml | SML |
|---------|-------|------|-------|-----|
| Ownership | ✓ | ✓ | ✗ | ✗ |
| Borrow checking | ✓ | ✓ | ✗ | ✗ |
| GC | ✗ | ✗ | ✓ | ✓ |
| Mutable refs | `&mut T` | `&mut T` | `ref` | `ref` |
| Immutable refs | `&T` | `&T` | implicit | implicit |
| Pattern matching | ✓ | ✓ | ✓ | ✓ |
| Type inference | ✓ | ✓ | ✓ | ✓ |
| Lifetime annotations | ✗ | ✓ | ✗ | ✗ |
| ML syntax | ✓ | ✗ | ✓ | ✓ |

### D. Implementation Notes

**Compiler Phases:**
1. **Parsing**: Standard ML parser with reference extensions
2. **Type inference**: Hindley-Milner with ownership tracking
3. **Borrow checking**: Infer borrows, check lifetimes
4. **Escape analysis**: Determine stack vs heap allocation
5. **Optimization**: Copy/move elision, tail calls
6. **Codegen**: Insert drop calls at scope exits

**Optimization Opportunities:**
- Copy elision for temporary values
- Move elision for return values  
- Small string optimization
- Tail call optimization (ownership-aware)
- Inline small functions
- Stack allocation when proven safe

---

## Conclusion

- **Clear reference syntax**: `&T` vs `&mut T` eliminates ambiguity
- **Minimal data structures**: Five core types cover all common use cases
- **Automatic allocation**: Escape analysis handles stack/heap decisions
- **Thread safety**: Ownership prevents data races at compile time
- **Simple mental model**: Immutable by default, mutable when explicit

Monad combines the best of functional and systems programming: the elegance of ML with the safety and performance of Rust, all without lifetime annotations or garbage collection.
---
