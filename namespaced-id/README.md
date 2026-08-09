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

# Examples

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

# Origin

The original idea for this library came from Minecraft's namespaced id, which is very very
similar. The game uses it for everything, and I came to like it a lot as I used it, so I made
this library to use it in my projects. Notice that it isn't a fully faithful reproduction,
though, as this library allows `/` characters in the namespace.
