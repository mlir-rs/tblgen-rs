#!/bin/sh

set -e

llvm_version=19

brew update
brew install llvm@$llvm_version

llvm_prefix=$(brew --prefix llvm@$llvm_version)

echo TABLEGEN_190_PREFIX=$llvm_prefix >>$GITHUB_ENV
echo PATH=$llvm_prefix/bin:$PATH >>$GITHUB_ENV
echo LIBRARY_PATH=$(brew --prefix)/lib:$LIBRARY_PATH >>$GITHUB_ENV
echo LD_LIBRARY_PATH=$(brew --prefix)/lib:$LD_LIBRARY_PATH >>$GITHUB_ENV
