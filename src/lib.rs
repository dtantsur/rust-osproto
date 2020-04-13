// Copyright 2019 Dmitry Tantsur <divius.inside@gmail.com>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Protocol Structures for OpenStack API
//!
//! # Introduction
//!
//! This crate is a collection of structures written by hand based on [OpenStack
//! API reference documentation](https://developer.openstack.org/api-ref/),
//! adapting it for more native Rust look and feel.
//!
//! This crate does not contain any code to access OpenStack API. For low-level
//! asynchronous SDK, check out [rust-osauth](https://crates.io/crates/osauth),
//! for a more high-level API see
//! [rust-openstack](https://crates.io/crates/openstack).
//!
//! # Stability
//!
//! This crate is unstable by design. Particularly, two kinds of breaking changes will be regularly
//! made:
//! 1. Adding new public fields to existing structures.
//! 2. Making required fields optional.
//!
//! Downstream crates as supposed to lock a single version of the crate and not expose its
//! structures as part of their public API.

#![crate_name = "osproto"]
#![crate_type = "lib"]
// NOTE: we do not use generic deny(warnings) to avoid breakages with new
// versions of the compiler. Add more warnings here as you discover them.
// Taken from https://github.com/rust-unofficial/patterns/
#![deny(
    const_err,
    dead_code,
    improper_ctypes,
    missing_copy_implementations,
    missing_debug_implementations,
    non_shorthand_field_patterns,
    no_mangle_generic_items,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    private_in_public,
    trivial_casts,
    trivial_numeric_casts,
    unconditional_recursion,
    unsafe_code,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_doc_comments,
    unused_import_braces,
    unused_parens,
    unused_qualifications,
    unused_results,
    while_true
)]
#![allow(missing_docs)]

pub mod common;
pub mod identity;
