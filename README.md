# mmd-rs

**THIS CRATE IS CURRENTLY A WORK IN PROGRESS AND NOT READY FOR USE!!!!**

A Rust parser for MikuMikuDance (MMD) model files.

## About

This crate aims to provide a safe, efficient parser for formats used by MikuMikuDance

## Example usage
```rust
use mmd_rs::pmx::Pmx;
    
let pmx = Pmx::open("path/to/model.pmx").expect("Failed to open PMX file");
println!("Model name: {}", pmx.header().name());
```

## Current Status

**Do not use this crate in production.** The API is unstable and major features are incomplete:

- [x] Basic PMX parsing structure
- [x] Vertex parsing
- [x] Material parsing
- [x] Texture parsing
- [x] Surface parsing
- [x] Bone support
- [x] Morph support
- [x] Display Frame support
- [x] Physics support (rigid bodies, joints)
- [ ] Soft body support (2.1)
- [ ] Documentation
- [ ] Examples
- [ ] Tests
- [ ] PMD support
- [ ] Serialization support

## License

MIT
