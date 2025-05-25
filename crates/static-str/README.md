# static-str

[`StaticStr`] - a string type that can handle both static and owned strings.

## Overview

The [`StaticStr`] type is designed to optimize string handling in scenarios where most strings are static
(known at compile time) but some need to be dynamically generated. It internally uses a [`Cow`] to avoid
unnecessary allocations when working with static strings while still maintaining the flexibility to handle
owned strings when needed.

## Use Cases

- Configuration values that are usually hardcoded but sometimes need to be generated
- Message templates with occasional dynamic content
- Any situation where you frequently use `&'static str` but occasionally need `String`

## Example

```rust
use static_str::StaticStr;

// Use with static strings - no allocation
let static_message: StaticStr = "Hello, World!".into();

// Use with owned strings - allocates only when needed
let dynamic_message: StaticStr = format!("Hello, {}!", "User").into();

// Both types can be used the same way
println!("{}", static_message);  // Hello, World!
println!("{}", dynamic_message); // Hello, User!
```

[`Cow`]: std::borrow::Cow

License: MPL-2.0
