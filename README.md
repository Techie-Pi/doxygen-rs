# doxygen-rs
Transform Doxygen to Rustdoc.

[**Docs available here**](https://techie-pi.github.io/doxygen-rs/doxygen_rs/) _Though somewhat outdated ;p_

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

## Architecture
Check [architecture.md](docs/architecture.md)
