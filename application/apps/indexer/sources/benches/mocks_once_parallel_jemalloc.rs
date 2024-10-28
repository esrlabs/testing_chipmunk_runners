//!Benchmarks for producer loop running multiple times in parallel with mock
//!parser and byte source using 'jemalloc' memory allocator.
//!
//!The mock of [`parsers::Parser`] will return [`std::iter::once()`] replicating the behavior of
//!the current built-in parsers in Chipmunk.

mod macros;

mocks_producer_once_parallel!(mocks_once_parallel_jemalloc, tikv_jemallocator::Jemalloc);