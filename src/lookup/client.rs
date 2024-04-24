#[cfg(not(feature = "blocking"))]
pub use ::reqwest::*;

#[cfg(feature = "blocking")]
pub use reqwest::blocking::*;
