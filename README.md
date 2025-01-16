# tblgen

[![GitHub Action](https://img.shields.io/github/actions/workflow/status/mlir-rs/tblgen-rs/test.yaml?branch=master&style=flat-square)](https://github.com/mlir-rs/tblgen-rs/actions?query=workflow%3Atest)
[![Crate](https://img.shields.io/crates/v/tblgen.svg?style=flat-square)](https://crates.io/crates/tblgen)
![Crates.io Total Downloads](https://img.shields.io/crates/d/tblgen)
![Crates.io License](https://img.shields.io/crates/l/tblgen)

Original project: https://gitlab.com/Danacus/tblgen-rs.

Original author: Daan Vanoverloop.

Thanks to the Daan for giving us access to publish to the original `tblgen` crate, we can now switch from the old `tblgen-alt` to `tblgen`. Future updates will be pushed to the original crate.

This crate provides raw bindings and a safe wrapper for TableGen, a domain-specific language used by the LLVM project.

The goal of this crate is to enable users to develop custom TableGen backends in Rust. Hence the primary use case of this crate are procedural macros that generate Rust code from TableGen description files.

## Documentation

Read the documentation at https://mlir-rs.github.io/tblgen-rs/tblgen/.

## Supported LLVM Versions

An installation of LLVM is required to use this crate. Both LLVM 16, 17, 18 and 19 are supported and can be selected using feature flags.

The `TABLEGEN_<version>_PREFIX` environment variable can be used to specify a custom directory of the LLVM installation.
