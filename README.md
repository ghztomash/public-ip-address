# 🔎 Public IP Address Lookup and Geolocation Information

<div align="center">
  
  [![Crates.io](https://img.shields.io/crates/v/public-ip-address.svg)](https://crates.io/crates/public-ip-address)
  [![Documentation](https://docs.rs/public-ip-address/badge.svg)](https://docs.rs/public-ip-address)
  ![cargo build](https://github.com/ghztomash/public-ip-address/actions/workflows/ci.yml/badge.svg)
  ![Crates.io License](https://img.shields.io/crates/l/public-ip-address)

</div>

![Demo](./assets/map_example.png)

`public-ip-address` is a simple Rust library for performing public IP lookups from over a dozen of various services.

It provides a unified interface to fetch public IP address and geolocation information from multiple providers.
Arbitrary IP address lookup and access API keys are supported for certain providers.
The library provides an asynchronous interface to make it easy to integrate with other `async` codebase.

The library also includes caching functionality to improve performance for repeated lookups
and minimize reaching rate-limiting thresholds.
The cache file can be encrypted when enabled through a feature flag for additional privacy.

## Usage

Add the following to your `Cargo.toml` file:
```toml
[dependencies]
public-ip-address = { version = "0.2", features = ["encryption"] }
```
## Example

The simplest way to use this library is to call the `perform_lookup()` function, which returns a `Result` with a `LookupResponse`.
```rust
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Perform my public IP address lookup
    let result = public_ip_address::perform_lookup(None).await?;
    println!("{}", result);
    Ok(())
}
```

More examples can be found in the `examples` directory. And run them with the following command:
```bash
cargo run --example <example_name>
```

## Providers

| Provider | URL | Rate Limit | API Key | Target Lookup |
| --- | --- | --- | --- | --- |
| FreeIpApi | [https://freeipapi.com](https://freeipapi.com) | 60 / minute | ✔️ | ✔️ |
| IfConfig | [https://ifconfig.co](https://ifconfig.co) | 1 / minute |  | ✔️ |
| IpInfo | [https://ipinfo.io](https://ipinfo.io) | 50000 / month | ✔️ | ✔️ |
| MyIp | [https://my-ip.io](https://my-ip.io) | ? / day | ️ | ️ |
| IpApiCom | [https://ip-api.com](https://ip-api.com) | 45 / minute |  | ✔️ |
| IpWhoIs | [https://ipwhois.io](https://ipwhois.io) | 10000 / month | ️ | ✔️ |
| IpApiCo | [https://ipapi.co](https://ipapi.co) | 30000 / month |  | ✔️ |
| IpApiIo | [https://ip-api.io](https://ip-api.io) | ? / day | ✔️ | ✔️ |
| IpBase | [https://ipbase.com](https://ipbase.com) | 10 / hour | ✔️ | ✔️ |
| IpLocateIo | [https://iplocate.io](https://iplocate.io) | 50 / day | ✔️ | ✔️ |
| IpLeak | [https://ipleak.net](https://ipleak.net) | ? / day | ️ | ✔️ |
| Mullvad | [https://mullvad.net](https://mullvad.net) | ? / day | ️ | ️ |
| AbstractApi | [https://abstractapi.com](https://abstractapi.com) | 1000 / day | ✔️ | ✔️ |
| IpGeolocation | [https://ipgeolocation.io](https://ipgeolocation.io) | 1000 / day | ✔️ | ✔️ |
| IpData | [https://ipdata.co](https://ipdata.co) | 1500 / day | ✔️ | ✔️ |
| Ip2Location | [https://ip2location.io](https://ip2location.io) | 500 / day (with key 30000 / month) | ✔️ | ✔️ |
| MyIpCom | [https://myip.com](https://myip.com) | unlimited | ️ | ️ |
| GetJsonIp | [https://getjsonip.com](https://getjsonip.com) | unlimited | ️ | ️ |
| Ipify | [https://www.ipify.org](https://www.ipify.org) | unlimited | ️ | ️ |

## Roadmap

- [x] Initial release
- [x] Add more providers
- [x] Add support for additional providers with API key
- [x] Add reverse lookup feature
- [ ] Add asynchronous and synchronous interface support
- [ ] Bulk lookup
- [ ] Offline reverse lookup

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
