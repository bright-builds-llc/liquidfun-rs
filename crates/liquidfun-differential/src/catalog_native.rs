//! Transactional public-API adapter for resolved native catalog plans.

mod capture;
mod executor;

pub use executor::NativeCatalogBackend;
