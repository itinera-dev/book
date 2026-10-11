In Rust, each field of the step marked `#[input]` is an input whose key is the field's name; `#[input("key")]` names another key. The field's type says whether it is optional: a field of type `Option<T>` is an optional input, `None` when the key is absent, and a field of any other type is required. On the initial data, a field of type `Option<T>` that is `None` leaves its key out of the data bag.

A value is any type that can be serialized, deserialized and cloned, such as `i64`, `String`, `bool` or a `Vec` of them. Types must match exactly: a price given as an `i32` and asked for as an `i64` is `wrong type`.

The step owns the values it receives, since each is its own clone, so changing one can never reach the data bag.
