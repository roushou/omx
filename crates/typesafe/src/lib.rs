//! Bounded relevance evaluation through TypeSafe's Jev HTTP API.

mod client;
mod evaluation;

pub use client::{Client, Error};
pub use evaluation::{Candidate, Ranking};
