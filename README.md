# doxygen-rs
Transform Doxygen to Rustdoc.

[**Docs available here**](https://techie-pi.github.io/doxygen-rs/doxygen_rs/)

## Installation
Add this to your ``Cargo.toml``
```toml
[dependencies]
doxygen-rs = { git = "https://github.com/Techie-Pi/doxygen-rs.git" }
```

## Example
```rust
use doxygen_rs::transform;

let rustdoc = transform("@brief Example Doxygen brief");
assert_eq!(rustdoc, "Example Doxygen brief\n\n");
```

## Structure
This repository is organized as follows:
- ``parser`` - The module in charge of taking _raw_ Boxygen comments and generating a list of values
- ``ast`` - The module in charge of taking the list of values and generating an AST
- ``generator`` - The module in charge of taking the AST and generating _raw_ Rustdoc comments

