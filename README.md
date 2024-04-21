[![CI Status]][workflow] [![MSRV]][repo] [![Latest Version]][crates.io] [![Rust Doc Crate]][docs.rs] [![Rust Doc Main]][docs]

[CI Status]: https://img.shields.io/github/actions/workflow/status/juntyr/wasi-logger/ci.yml?branch=main
[workflow]: https://github.com/juntyr/wasi-logger/actions/workflows/ci.yml?query=branch%3Amain

[MSRV]: https://img.shields.io/badge/MSRV-1.65.0-blue
[repo]: https://github.com/juntyr/wasi-logger

[Latest Version]: https://img.shields.io/crates/v/wasi-logger
[crates.io]: https://crates.io/crates/wasi-logger

[Rust Doc Crate]: https://img.shields.io/docsrs/wasi-logger
[docs.rs]: https://docs.rs/wasi-logger/

[Rust Doc Main]: https://img.shields.io/badge/docs-main-blue
[docs]: https://juntyr.github.io/wasi-logger/wasi_logger

# wasi-logger

`wasi-logger` provides a [`Logger`] implementing the [`log::Log`] logging API, which is backed by the [`wasi:logging/logging`] WIT interface.

## Usage

To use the [`Logger`] as a logger, it first needs to be installed once in the top-level WASM component using [`Logger::install`], e.g. in a `main` function, a ctor, or using a [`std::sync::OnceLock`]. Remember to also set the global logging max level using [`log::set_max_level`] to ensure that log entries created with [`log::log!`] and others are actually recorded.

```rust
#[macro_use]
extern crate log;

extern crate wasi_logger;

fn main() {
    wasi_logger::Logger::install().expect("failed to install wasi_logger::Logger");
    log::set_max_level(log::LevelFilter::Info);

    error!("Something went really wrong");
    info!("This is good to know");
    debug!("This message is not recorded as the trace level is currently disabled");
}
```

## Features

* The `kv` feature transitively enables `log/kv` and includes the key-value pairs in a log record in the log message.

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

## Funding

`wasi-logger` has been developed as part of [ESiWACE3](https://www.esiwace.eu), the third phase of the Centre of Excellence in Simulation of Weather and Climate in Europe.

Funded by the European Union. This work has received funding from the European High Performance Computing Joint Undertaking (JU) under grant agreement No 101093054.

[`Logger`]: https://docs.rs/wasi-logger/0.1/wasi_logger/struct.Logger.html
[`Logger::install`]: https://docs.rs/wasi-logger/0.1/wasi_logger/struct.Logger.html#method.install
[`log::log!`]: https://docs.rs/log/0.4/log/macro.log.html
[`log::Log`]: https://docs.rs/log/0.4/log/trait.Log.html
[`log::set_max_level`]: https://docs.rs/log/0.4/log/fn.set_max_level.html
[`std::sync::OnceLock`]: https://doc.rust-lang.org/std/sync/struct.OnceLock.html
[`wasi:logging/logging`]: https://github.com/WebAssembly/wasi-logging/blob/3293e84de91a1ead98a1b4362f95ac8af5a16ddd/wit/logging.wit
