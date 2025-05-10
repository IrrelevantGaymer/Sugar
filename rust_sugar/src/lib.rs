#![allow(incomplete_features)]
#![allow(unused_labels)]
#![feature(const_mut_refs)]
#![feature(const_trait_impl)]
#![feature(exact_size_is_empty)]
#![feature(iter_next_chunk)]
#![feature(iterator_try_collect)]
#![feature(let_chains)]
#![feature(never_type)]
#![feature(ptr_as_ref_unchecked)]
#![feature(slice_ptr_get)]
#![feature(strict_provenance)]
#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]
#![feature(try_trait_v2_yeet)]

pub mod compiler;
pub mod interpreter;
pub mod lexer;
pub mod parser;
pub mod full_result;
pub mod string_utils;
pub mod term;