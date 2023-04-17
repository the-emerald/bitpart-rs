//! BitPart is an exact search algorithm for high dimensional metric spaces. [See the paper here.](https://research-repository.st-andrews.ac.uk/handle/10023/21368)
//! # Example
//! ```
//! # use rand::prelude::*;
//! # use bitpart::metric::{Euclidean, Metric};
//! # use bitpart::{BitPart, BitPartBuilder};
//! #
//! let points: Vec<Euclidean<Vec<f64>>> = (0..1000)
//!     .map(|_| (0..20).map(|_| rand::random()).collect())
//!     .map(Euclidean::new)
//!     .collect();
//!
//! let bitpart = BitPartBuilder::new(points.clone(), 40).build();
//!
//! let query = points[0].clone();
//! let threshold = 0.1234;
//!
//! let res = bitpart.range_search(query.clone(), threshold);
//!
//! // All points are within the threshold.
//! assert!(res.iter().all(|(p, _)| p.distance(&query) <= threshold));
//!
//! // Results match a linear search.
//! let linear = points
//!     .into_iter()
//!     .map(|pt| pt.distance(&query))
//!     .filter(|d| *d <= threshold)
//!     .count();
//! assert_eq!(res.len(), linear);
//! ```
//! TODO: Talk about how the algorithm works here
//!
//! TODO: Talk about features here
//!
//! TODO: Talk about bitpart-fast-math and vectorisation

#![deny(missing_docs)]

mod builder;
pub use builder::*;

pub mod exclusions;
pub mod metric;

mod sequential;
pub use sequential::*;

#[cfg(feature = "rayon")]
mod parallel;
#[cfg(feature = "rayon")]
pub use parallel::*;

#[cfg(feature = "on_disk")]
mod on_disk;
#[cfg(feature = "on_disk")]
pub use on_disk::*;

/// Trait for BitPart data structures.
pub trait BitPart<T> {
    /// Perform a range search, given a `point` and a radius `threshold` around it.
    ///
    /// Returns a vector of points which fall within the specific radius, along with their distance from the query `point`.
    fn range_search(&self, point: T, threshold: f64) -> Vec<(T, f64)>;
}
