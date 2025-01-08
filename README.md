# tblgen

[![GitHub Action](https://img.shields.io/github/actions/workflow/status/edg-l/tblgen-rs-alt/test.yaml?branch=master&style=flat-square)](https://github.com/edg-l/tblgen-rs-alt/actions?query=workflow%3Atest)
[![Crate](https://img.shields.io/crates/v/tblgen-alt.svg?style=flat-square)](https://crates.io/crates/tblgen-alt)
![Crates.io Total Downloads](https://img.shields.io/crates/d/tblgen-alt)
![Crates.io License](https://img.shields.io/crates/l/tblgen-alt)

This is a updated (LLVM 19) fork of https://gitlab.com/Danacus/tblgen-rs.
Original author: Daan Vanoverloop.

This crate provides raw bindings and a safe wrapper for TableGen, a domain-specific language used by the LLVM project.

The goal of this crate is to enable users to develop custom TableGen backends in Rust. Hence the primary use case of this crate are procedural macros that generate Rust code from TableGen description files.

## Documentation

Read the documentation at https://mlir-rs.github.io/tblgen-rs-alt/tblgen_alt/.

## Supported LLVM Versions

An installation of LLVM is required to use this crate. Both LLVM 16, 17 and 18 are supported and can be selected using feature flags.

The `TABLEGEN_<version>_PREFIX` environment variable can be used to specify a custom directory of the LLVM installation.
