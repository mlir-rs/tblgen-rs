#!/bin/sh

set -e

llvm_version=${LLVM_VERSION:-22}

brew update
brew install llvm@$llvm_version zstd

llvm_prefix=$(brew --prefix llvm@$llvm_version)
zstd_prefix=$(brew --prefix zstd)

echo TABLEGEN_${llvm_version}0_PREFIX=$llvm_prefix >>$GITHUB_ENV
echo PATH=$llvm_prefix/bin:$PATH >>$GITHUB_ENV
echo LIBRARY_PATH=$llvm_prefix/lib:$zstd_prefix/lib:$LIBRARY_PATH >>$GITHUB_ENV
echo LD_LIBRARY_PATH=$llvm_prefix/lib:$zstd_prefix/lib:$LD_LIBRARY_PATH >>$GITHUB_ENV
