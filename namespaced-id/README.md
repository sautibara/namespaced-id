# Namespaced Id

[![crates.io](https://img.shields.io/crates/v/namespaced-id.svg)](https://crates.io/crates/namespaced-id)
[![docs.rs](https://docs.rs/namespaced-id/badge.svg)](https://docs.rs/namespaced-id)
[![License](https://img.shields.io/crates/l/namespaced-id)](https://github.com/sautibara/namespaced-id)
[![brainmade.org](https://img.shields.io/badge/brainmade.org-FFFFFF?style=social&logo=data%3Aimage%2Fsvg%2Bxml%3Bbase64%2CPHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSIxZW0iIGhlaWdodD0iNzkiIHZpZXdCb3g9IjAgMCA2NyA3OSIgZmlsbD0ibm9uZSI%2BPHBhdGggZmlsbD0iIzAwMCIgZD0iTTUyLjYxMiA3OC43ODJIMjMuMzNhMi41NTkgMi41NTkgMCAwIDEtMi41Ni0yLjU1OHYtNy42NzdoLTcuOTczYTIuNTYgMi41NiAwIDAgMS0yLjU2LTIuNTZWNTUuMzE1bC04LjgyLTQuMzk3YTIuNTU5IDIuNTU5IDAgMCAxLS45ODYtMy43MWw5LjgwNy0xNC43MTR2LTQuMzVDMTAuMjQgMTIuNTk5IDIyLjg0MyAwIDM4LjM4OCAwIDUzLjkzMiAwIDY2LjUzNCAxMi42IDY2LjUzOCAyOC4xNDNjLS42MzIgMjcuODI0LTEwLjc2IDIzLjUxNi0xMS4xOCAzNC4wNDVsLS4xODcgMTQuMDM1YTIuNTkgMi41OSAwIDAgMS0uNzUgMS44MSAyLjU1IDIuNTUgMCAwIDEtMS44MDkuNzVabS0yNi43MjMtNS4xMTdoMjQuMTY0bC4yODYtMTQuNTQyYy0uMjYzLTYuNjU2IDExLjcxNi04LjI0MyAxMS4wOC0zMC43MzQtLjM1OC0xMi43MTMtMTAuMzEzLTIzLjI3MS0yMy4wMzEtMjMuMjcxLTEyLjcxOCAwLTIzLjAyOSAxMC4zMDctMjMuMDMyIDIzLjAyNXY1LjExN2MwIC41MDYtLjE1IDEtLjQzIDEuNDJsLTguNjMgMTIuOTQxIDcuNjQ1IDMuODJhMi41NTkgMi41NTkgMCAwIDEgMS40MTUgMi4yOTF2OS42OTdoNy45NzRhMi41NTkgMi41NTkgMCAwIDEgMi41NiAyLjU1OXY3LjY3N1oiLz48cGF0aCBmaWxsPSIjMDAwIiBkPSJNNDAuMzcyIDU4LjIyMlYzOC45MzRjLjExOCAwIC4yMzcuMDE4LjM1NS4wMTggOS43NjktLjAxMiAxNy4wNS05LjAxMiAxNS4wMjItMTguNTY3YTIuMzY2IDIuMzY2IDAgMCAwLTEuODIxLTEuODIyYy04LjEwNi0xLjczLTE2LjEyMSAzLjI5Mi0xOC4wOTggMTEuMzQxLS4wMjQtLjAyNC0uMDQzLS4wNS0uMDY2LS4wNzNhMTUuMzIzIDE1LjMyMyAwIDAgMC0xNC4wNi00LjE3IDIuMzY1IDIuMzY1IDAgMCAwLTEuODIxIDEuODJjLTIuMDI4IDkuNTU1IDUuMjUyIDE4LjU1NCAxNS4wMiAxOC41NjguMjM2IDAgLjQ5Mi0uMDI4LjczOC0uMDR2MTIuMjEzaDQuNzMxWm0yLjgzOS0zMi4xNDNhMTAuNjQ2IDEwLjY0NiAwIDAgMSA4LjEyNC0zLjEwNmMuMzUgNi4zNC00Ljg4OCAxMS41NzctMTEuMjI4IDExLjIzYTEwLjU4IDEwLjU4IDAgMCAxIDMuMTA0LTguMTI0Wk0yNy40MDMgMzguMTkzYTEwLjYwNyAxMC42MDcgMCAwIDEtMy4xMTgtOC4xMjNjNi4zNDQtLjM1OCAxMS41ODcgNC44ODYgMTEuMjI4IDExLjIzLTMuMDIzLjE2OS01Ljk3My0uOTYxLTguMTEtMy4xMDdaIi8%2BPC9zdmc%2B)](https://brainmade.org/)

A crate that defines types that identify data in a human-readable way.

The most common example is the [`NamespacedId`], which consists of a namespace and path
separated by a colon (ex: `namespace:path`). This leads to an id that is somewhat resistant to
collisions, but is still human-readable.

All ids are based on a [`DelimitedId`], which is a generic id with a constant number of
components separated by colons. It is a wrapper over a [`Box<str>`], ensuring that its interior
is a valid id. Every id also has a corresponding wrapper over a [`str`], backed by a
[`DelimitedIdRef`]. As both are wrappers over `Box<str>` and `str` respectively, they can be
losslessly converted between the two without having to allocate.

There are three main id types:
- [`IdComponent`] (ex: `component`)
    - Used to ensure that the backing string is valid as a single component of a larger id
      (no separators or invalid characters).
- [`NamespacedId`] (ex: `namespace:path`)
    - Used to identify different objects.
- [`OperationId`] (ex: `namespace:path:operation`)
    - Used to identify operations that refer to a specific identified object.

Each main id type also has a corresponding macro to generate and validate a static reference at
compile time.
- `IdComponent` has [`ident_component!`]
- `NamespacedId` has [`ident!`]
- `OperationId` has [`op_ident!`]

## Examples

```rust
use namespaced_id::ident;

let id = ident!("namespace:path");
assert_eq!("namespace", id.namespace());
assert_eq!("path", id.path());
```

```rust
use namespaced_id::NamespacedIdRef;

let runtime_str = "other:id";
let id = NamespacedIdRef::new(runtime_str)
    .expect("runtime_str is a valid id");

assert_eq!("other", id.namespace());
assert_eq!("id", id.path());
```

## Origin

The original idea for this library came from Minecraft's namespaced id, which is very very
similar. The game uses it for everything, and I came to like it a lot as I used it, so I made
this library to use it in my projects. Notice that it isn't a fully faithful reproduction,
though, as this library allows `/` characters in the namespace.
