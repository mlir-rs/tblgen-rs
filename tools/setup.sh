#!/bin/sh

set -e

llvm_version=20

brew update
brew install llvm@$llvm_version

llvm_prefix=$(brew --prefix llvm@$llvm_version)

echo TABLEGEN_200_PREFIX=$llvm_prefix >>$GITHUB_ENV
echo PATH=$llvm_prefix/bin:$PATH >>$GITHUB_ENV
echo LIBRARY_PATH=$llvm_prefix/lib:$LIBRARY_PATH >>$GITHUB_ENV
echo LD_LIBRARY_PATH=$llvm_prefix/lib:$LD_LIBRARY_PATH >>$GITHUB_ENV
