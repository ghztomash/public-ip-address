![cargo build](https://github.com/ghztomash/public-ip-address/actions/workflows/ci.yml/badge.svg)

# 🔎 Public IP Address Lookup and Geolocation Information

![Demo](./assets/map_example.png)

`public-ip-address` is a simple Rust library for performing public IP lookups from various services.

It provides a unified interface to fetch public IP address and geolocation information from multiple providers.

The library also includes caching functionality to improve performance for repeated lookups and minimaze rate-limiting.

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

More examplse can be found in the `examples` directory. And run them with the following command:
```bash
cargo run --example <example_name>
```

## Providers

| Provider | URL |
| --- | --- |
| FreeIpApi | [https://freeipapi.com](https://freeipapi.com) |
| IfConfig | [https://ifconfig.co](https://ifconfig.co) |
| IpInfo | [https://ipinfo.io](https://ipinfo.io) |
| MyIp | [https://my-ip.io](https://my-ip.io) |
| IpApiCom | [https://ip-api.com](https://ip-api.com) |
| IpWhoIs | [https://ipwhois.io](https://ipwhois.io) |
| IpApiCo | [https://ipapi.co](https://ipapi.co) |
| IpApiIo | [https://ip-api.io](https://ip-api.io) |
| IpBase | [https://ipbase.com](https://ipbase.com) |

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Contributions are welcome! Please submit a pull request.

## Support

If you encounter any problems or have any questions, please open an issue in the GitHub repository.
