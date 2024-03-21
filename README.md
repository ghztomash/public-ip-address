# Public IP address

This package provides a library for performing public IP lookups from various services.

## Usage

Add the following to your `Cargo.toml` file:
```toml
[dependencies]
public-ip-address = { version = "0.1" }
```
## Example

The simplest way to use this library is to call the `perform_lookup()` function, which returns a `Result` with a `LookupResponse`.
```rust
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let result = public_ip_address::perform_lookup()?;
    println!("{}", result);
    Ok(())
}
```

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
