Protocol Structures for OpenStack API
=====================================

[![Build
Status](https://travis-ci.org/dtantsur/rust-osproto.svg?branch=master)](https://travis-ci.org/dtantsur/rust-osproto)
[![License](https://img.shields.io/crates/l/osproto.svg)](https://github.com/dtantsur/rust-osproto/blob/master/LICENSE)
[![Latest
Version](https://img.shields.io/crates/v/osproto.svg)](https://crates.io/crates/osproto)
[![Documentation](https://img.shields.io/badge/documentation-latest-blueviolet.svg)](https://docs.rs/osproto)

This crate is a collection of structures written by hand based on [OpenStack
API reference documentation](https://developer.openstack.org/api-ref/),
adapting it for more native Rust look and feel.

This crate does not contain any code to access OpenStack API. For low-level
asynchronous SDK, check out [rust-osauth](https://crates.io/crates/osauth),
for a more high-level API see
[rust-openstack](https://crates.io/crates/openstack).
