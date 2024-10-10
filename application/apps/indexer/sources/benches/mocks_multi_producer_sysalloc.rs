//! Benchmarks for producer loop with mock parser and byte source using rust standard memory allocator,
//! which provides the best performance and is used in Chipmunk app currently.
//!
//! The mock of [`parsers::Parser`] will return iterator with multiple value replicating the
//! behavior of the potential plugins in Chipmunk.

mod macros;

mocks_producer_multi!(mocks_multi_producer_sysalloc, std::alloc::System);