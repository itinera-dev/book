Services are not in itinera yet, so this chapter's example is not compiled; it is written against the planned API, which may still change.

In Rust, a service is a trait object, such as `dyn StockStore`, and the trait is `Send + Sync`. A step asks for one as a parameter of `run` marked `#[service]`, of type `&dyn StockStore`; the key is the parameter's name, or the one `#[service("key")]` names. The reference borrows the attempt, so the step cannot keep the service once its attempt ends, and its struct needs no lifetime.

A provider is a method of the workflow's own `impl` block, marked `#[provider]` for one instance per journey or `#[provider(per_attempt)]` for one per attempt, with `steps = ["reserve", "ship"]` to attach it to those steps only. Its key is the method's name, unless `key = "..."` names it. It returns `Result<Box<dyn StockStore>, Error>`, and an `Err` is a provider that fails. Since it takes `&self`, it can use the workflow's own fields, such as a connection pool. Each instance is dropped when its scope ends, so a client that must close does so in its `Drop`.

The macros make every mistake they can see a compile error: a step asking for a key the workflow does not provide, two providers of one key serving the same step, and a provider whose type differs from what a step asks for.
