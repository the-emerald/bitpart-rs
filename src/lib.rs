pub mod builder;
pub mod exclusions;
pub mod metric;

mod sequential;
pub use sequential::*;

#[cfg(feature = "rayon")]
pub mod parallel;
