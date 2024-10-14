#![allow(incomplete_features)]
#![allow(unused_labels)]
#![feature(const_mut_refs)]
#![feature(iterator_try_collect)]
#![feature(let_chains)]
#![feature(never_type)]
#![feature(ptr_as_ref_unchecked)]
#![feature(strict_provenance)]
#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]
#![feature(try_trait_v2_yeet)]

// #[path = "../assets/crates/dashmap/src/lib.rs"]
// pub mod dashmap;

pub mod lexer;
pub mod parser;
pub mod full_result;
pub mod string_utils;