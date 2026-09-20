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
The library provides an asynchronous and blocking interfaces to make it easy to integrate with other `async` codebase.

The library also includes caching functionality to improve performance for repeated lookups
and minimize reaching rate-limiting thresholds.
The cache file can be encrypted when enabled through a feature flag for additional privacy.

## Usage

Add the following to your `Cargo.toml` file:

```toml
[dependencies]
public-ip-address = { version = "0.4" }

# with cache encryption enabled
public-ip-address = { version = "0.4", features = ["encryption"] }

# with `async` disabled
public-ip-address = { version = "0.4", features = ["blocking"] }

# for targets that do not support openssl
public-ip-address = { version = "0.4", default-features = false, features = ["rustls-tls"] }
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

With `blocking` interface enabled:

```rust
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // Perform my public IP address lookup
    let result = public_ip_address::perform_lookup(None)?;
    println!("{}", result);
    Ok(())
}
```

More examples can be found in the `examples` directory. And run them with the following command:

```bash
cargo run --example <example_name>
```

Running the examples with the `blocking` feature enabled:

```bash
cargo run --example <example_name> --features blocking
```

## Using your own HTTP client

By default, `perform_lookup` and friends build their own `reqwest::Client` for each request. If you need control over the transport — a proxy, an explicit `no_proxy()`, custom timeouts, or a shared connection pool — build a client yourself and pass it to `perform_lookup_with_client`:

```rust
use std::error::Error;
use public_ip_address::lookup::{Client, LookupProvider};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Ignore any proxy configured in the environment or by the OS, and cap the timeout.
    let client = Client::builder()
        .no_proxy()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    let providers = vec![
        // List of providers to use for the lookup
        // (LookupProvider::IpWhoIs, Some(Parameters::new(apikey)))
    ];

    let result = public_ip_address::perform_lookup_with_client(&client, providers, None).await?;
    println!("{}", result);
    Ok(())
}
```

The same client can also be given to a single provider through `LookupService::with_client`.

With the `blocking` feature enabled the example above drops `#[tokio::main]` and `.await`, exactly like the other examples. The re-exported `Client`, `RequestBuilder`, and `Response` types come from `reqwest`, and follow the build: they are the asynchronous types by default and the `reqwest::blocking` ones under the `blocking` feature.

Note that the cached helpers, such as `perform_lookup` and `perform_cached_lookup_with`, always use a default client; only `perform_lookup_with_client` accepts a caller-supplied one. A caller that wants both can drive the cache directly through `public_ip_address::cache::ResponseCache`.

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
| Ip2Location | [https://ip2location.io](https://ip2location.io) | 50000 / month | ✔️ | ✔️ |
| MyIpCom | [https://myip.com](https://myip.com) | unlimited | ️ | ️ |
| GetJsonIp | [https://getjsonip.com](https://getjsonip.com) | unlimited | ️ | ️ |
| Ipify | [https://www.ipify.org](https://www.ipify.org) | unlimited | ️ | ️ |
| IpQuery | [https://ipquery.io](https://ipquery.io) | unlimited |  | ✔️ |

## Roadmap

- [x] Initial release
- [x] Add more providers
- [x] Add support for additional providers with API key
- [x] Add reverse lookup feature
- [x] Add asynchronous and synchronous interface support
- [ ] Bulk lookup
- [ ] Offline reverse lookup

## License

Licensed under either of

- Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license
   ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## Contribution

Contributions are welcome! Please submit a pull request.

## Support

If you encounter any problems or have any questions, please open an issue in the GitHub repository.
