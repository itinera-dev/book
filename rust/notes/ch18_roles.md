In Rust, a role is a trait, and a hook asks for it with a parameter marked `#[role]` of type `&dyn Messages`. The workflow provides it by implementing the trait, with `#[role]` on the `impl` block, which makes the workflow hand itself to every hook that asks for that role. The trait is therefore dyn compatible. Attaching a policy to a workflow that does not provide one of its roles does not compile.

The workflow's fields, such as `channel`, are the workflow's own configuration, never journey data. The instance builder has one setter for each, and `create` does not compile until every field that is neither an `Option` nor marked `#[default]` has been set.

In an asynchronous hook, the role is held across `.await`, so its trait is declared `trait Messages: Sync`.
