#![cfg_attr(not(doc), no_std)]
#![deny(unsafe_code)]
#![forbid(unstable_features)]
#![deny(clippy::complexity)]
#![deny(clippy::correctness)]
#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]
#![deny(clippy::perf)]
#![deny(clippy::style)]
#![deny(clippy::suspicious)]
#![deny(clippy::cargo)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::indexing_slicing)]
#![warn(clippy::panic)]
#![warn(clippy::todo)]
#![warn(clippy::unimplemented)]
#![warn(clippy::unreachable)]

//! [![CI Status]][workflow] [![MSRV]][repo] [![Latest Version]][crates.io]
//! [![Rust Doc Crate]][docs.rs] [![Rust Doc Main]][docs]
//!
//! [CI Status]: https://img.shields.io/github/actions/workflow/status/juntyr/wasi-logger/ci.yml?branch=main
//! [workflow]: https://github.com/juntyr/wasi-logger/actions/workflows/ci.yml?query=branch%3Amain
//!
//! [MSRV]: https://img.shields.io/badge/MSRV-1.65.0-blue
//! [repo]: https://github.com/juntyr/wasi-logger
//!
//! [Latest Version]: https://img.shields.io/crates/v/wasi-logger
//! [crates.io]: https://crates.io/crates/wasi-logger
//!
//! [Rust Doc Crate]: https://img.shields.io/docsrs/wasi-logger
//! [docs.rs]: https://docs.rs/wasi-logger/
//!
//! [Rust Doc Main]: https://img.shields.io/badge/docs-main-blue
//! [docs]: https://juntyr.github.io/wasi-logger/wasi_logger
//!
//! # wasi-logger
//!
//! `wasi-logger` provides a [`Logger`] implementing the [`log::Log`] logging
//! API, which is backed by the [`wasi:logging/logging`] WIT interface.
//!
//! ## Usage
//!
//! To use the [`Logger`] as a logger, it first needs to be installed once in
//! the top-level WASM component using [`Logger::install`], e.g. in a `main`
//! function, a ctor, or using a [`std::sync::OnceLock`]. Remember to also set
//! the global logging max level using [`log::set_max_level`] to ensure that log
//! entries created with [`log::log!`] and others are actually recorded.
//!
//! ```rust,ignore
//! #[macro_use]
//! extern crate log;
//!
//! extern crate wasi_logger;
//!
//! fn main() {
//!     wasi_logger::Logger::install().expect("failed to install wasi_logger::Logger");
//!     log::set_max_level(log::LevelFilter::Info);
//!
//!     error!("Something went really wrong");
//!     info!("This is good to know");
//!     debug!("This message is not recorded as the trace level is currently disabled");
//! }
//! ```
//!
//! ## Features
//!
//! * The `kv` feature transitively enables `log/kv` and includes the key-value
//!   pairs in a log record in the log message.
//!
//! [`wasi:logging/logging`]: https://github.com/WebAssembly/wasi-logging/blob/3293e84de91a1ead98a1b4362f95ac8af5a16ddd/wit/logging.wit

extern crate alloc;

use core::fmt::Write;

use log::{Level, Log, Metadata, Record, SetLoggerError};

use crate::bindings::wasi::logging::logging::{log, Level as LoggingLevel};

#[allow(clippy::missing_safety_doc)]
mod bindings {
    wit_bindgen::generate!("crates-io:wasi-logger/crate");
}

/// `Logger` which implements [`log::Log`] and is backed by the
/// [`wasi:logging/logging`] WIT interface.
///
/// [`wasi:logging/logging`]: https://github.com/WebAssembly/wasi-logging/blob/3293e84de91a1ead98a1b4362f95ac8af5a16ddd/wit/logging.wit
pub struct Logger {
    _private: (),
}

impl Logger {
    /// Sets the global logger to a `Logger` using [`log::set_logger`] and
    /// returns the installed `Logger`.
    ///
    /// This function may only be called once in the lifetime of a program. Any
    /// log events that occur before the call to [`log::set_logger`]
    /// completes will be ignored.
    ///
    /// # Errors
    ///
    /// An error is returned if a logger has already been set.
    pub fn install() -> Result<&'static Self, SetLoggerError> {
        static LOGGER: Logger = Logger { _private: () };

        log::set_logger(&LOGGER)?;

        Ok(&LOGGER)
    }
}

impl Log for Logger {
    #[inline]
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    #[inline]
    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let level = match record.level() {
            Level::Error => LoggingLevel::Error,
            Level::Warn => LoggingLevel::Warn,
            Level::Info => LoggingLevel::Info,
            Level::Debug => LoggingLevel::Debug,
            Level::Trace => LoggingLevel::Trace,
        };

        if let Some(message) = record.args().as_str() {
            if record.module_path().is_none()
                && record.file().is_none()
                && record.line().is_none()
                && {
                    #[cfg(feature = "kv")]
                    {
                        record.key_values().count() == 0
                    }
                    #[cfg(not(feature = "kv"))]
                    {
                        true
                    }
                }
            {
                return log(level, record.target(), message);
            }
        }

        let mut message = alloc::string::String::new();

        if let Some(module_path) = record.module_path() {
            if module_path != record.target() {
                message.push_str(module_path);
            }
        }

        if let Some(file) = record.file() {
            if !message.is_empty() {
                message.push_str(" in ");
            }

            message.push_str(file);
        }

        if let Some(line) = record.line() {
            if !message.is_empty() {
                message.push(':');
            }

            #[allow(clippy::unwrap_used)]
            // formatting a u32 cannot fail
            message.write_fmt(format_args!("{line}")).unwrap();
        }

        if !message.is_empty() {
            message.push_str(": ");
        }

        #[allow(clippy::expect_used)]
        // failing to format the args is a bug and we cannot continue
        message
            .write_fmt(*record.args())
            .expect("formatting log::Record::args() returned an error");

        #[cfg(feature = "kv")]
        {
            if record.key_values().count() > 0 {
                if !message.is_empty() {
                    message.push(' ');
                }

                #[allow(clippy::expect_used)]
                // failing to format the key-value pairs is a bug and we cannot continue
                message
                    .write_fmt(format_args!("{:?}", KeyValues(record.key_values())))
                    .expect("debug-formatting log::Record::key_values() returned an error");
            }
        }

        log(level, record.target(), &message);
    }

    #[inline]
    fn flush(&self) {}
}

#[cfg(feature = "kv")]
struct KeyValues<'a>(&'a dyn log::kv::Source);

#[cfg(feature = "kv")]
impl<'a> core::fmt::Debug for KeyValues<'a> {
    // adapted from log v0.4.21
    // released under the MIT or Apache 2.0 License
    // https://docs.rs/log/0.4.21/src/log/lib.rs.html#730-745
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
        let mut visitor = fmt.debug_map();
        self.0.visit(&mut visitor).map_err(|_| core::fmt::Error)?;
        visitor.finish()
    }
}
