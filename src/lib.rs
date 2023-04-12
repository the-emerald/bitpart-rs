//! BitPart is an exact search algorithm for high dimensional metric spaces. [See the paper here.](https://research-repository.st-andrews.ac.uk/handle/10023/21368)
//! # Example
//! ```
//! # use rand::prelude::*;
//! # use bitpart::metric::euclidean::Euclidean;
//! # use bitpart::builder::BitPartBuilder;
//! # use bitpart::metric::Metric;
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
//! TODO: Talk about features here
//!
//! TODO: Talk about bitpart-fast-math and vectorisation

/// Builder struct for the data structure
pub mod builder;
pub mod exclusions;
/// Metric trait definitions
pub mod metric;

mod sequential;
pub use sequential::*;

/// Parallel BitPart
#[cfg(feature = "rayon")]
pub mod parallel;

/// On-disk BitPart
#[cfg(feature = "on_disk")]
pub mod on_disk;
