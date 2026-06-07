//! ares-scanner — Project scanning engine using Tree-sitter.
//!
//! Parses source files into graph nodes and edges.
//! Supports: TypeScript, Python, Go (Week 6-7).

pub mod delta;
pub mod extractor;
pub mod hasher;
pub mod parser;
pub mod scanner;
pub mod watcher;

pub use scanner::Scanner;
pub use hasher::hash_file;
