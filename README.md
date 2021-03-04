# vpp-api-gen

This is a work-in-progress repo for low-level VPP API in Rust.

At present all it does is load the .api.json files, no codegen yet.

Useful incantations:
```
cargo test
cargo run -- --in-file testdata/ --parse-type Tree --print-message-names
```
