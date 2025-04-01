// Original work Copyright 2016 Alexander Stocko <as@coder.gg>.
// Modified work Copyright 2023 Daan Vanoverloop
// See the COPYRIGHT file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#ifndef _CTABLEGEN_TABLEGEN_HPP_
#define _CTABLEGEN_TABLEGEN_HPP_

#include <memory>
#include <utility>

#include <llvm/Support/CommandLine.h>
#include <llvm/Support/FileSystem.h>
#include <llvm/Support/MemoryBuffer.h>
#include <llvm/Support/ToolOutputFile.h>
#include <llvm/TableGen/Error.h>
#include <llvm/TableGen/Parser.h>
#include <llvm/TableGen/Record.h>

#include "TableGen.h"
#include "Types.h"
#include "llvm/Support/CBindingWrapping.h"

using namespace llvm;

namespace ctablegen {

typedef std::map<std::string, std::unique_ptr<Record>, std::less<>> RecordMap;
typedef std::vector<const Record *> RecordVector;
typedef std::pair<std::string, TypedInit *> DagPair;

class TableGenParser {
public:
  TableGenParser() {}
  bool addSource(const char *source);
  void addSourceFile(const StringRef source);
  void addIncludeDirectory(const StringRef include);
  RecordKeeper *parse();

  SourceMgr sourceMgr;

private:
  std::vector<std::string> includeDirs;
  std::vector<std::string> files;
};

// Utility
TableGenRecTyKind tableGenFromRecType(const RecTy *rt);

/// A simple raw ostream subclass that forwards write_impl calls to the
/// user-supplied callback together with opaque user-supplied data.
class CallbackOstream : public llvm::raw_ostream {
public:
  CallbackOstream(std::function<void(TableGenStringRef, void *)> callback,
                  void *opaqueData)
      : raw_ostream(/*unbuffered=*/true), callback(std::move(callback)),
        opaqueData(opaqueData), pos(0u) {}

  void write_impl(const char *ptr, size_t size) override {
    TableGenStringRef string = TableGenStringRef{.data = ptr, .len = size};
    callback(string, opaqueData);
    pos += size;
  }

  uint64_t current_pos() const override { return pos; }

private:
  std::function<void(TableGenStringRef, void *)> callback;
  void *opaqueData;
  uint64_t pos;
};

} // namespace ctablegen

DEFINE_SIMPLE_CONVERSION_FUNCTIONS(ctablegen::TableGenParser,
                                   TableGenParserRef);
DEFINE_SIMPLE_CONVERSION_FUNCTIONS(RecordKeeper, TableGenRecordKeeperRef);

DEFINE_SIMPLE_CONVERSION_FUNCTIONS(ctablegen::RecordMap, TableGenRecordMapRef);
DEFINE_SIMPLE_CONVERSION_FUNCTIONS(ctablegen::RecordVector,
                                   TableGenRecordVectorRef);
DEFINE_SIMPLE_CONVERSION_FUNCTIONS(ArrayRef<Record>, TableGenRecordArrayRef);
DEFINE_SIMPLE_CONVERSION_FUNCTIONS(ArrayRef<RecordVal>,
                                   TableGenRecordValArrayRef);

DEFINE_SIMPLE_CONVERSION_FUNCTIONS(Record, TableGenRecordRef);
DEFINE_SIMPLE_CONVERSION_FUNCTIONS(RecordVal, TableGenRecordValRef);

DEFINE_SIMPLE_CONVERSION_FUNCTIONS(TypedInit, TableGenTypedInitRef);
DEFINE_SIMPLE_CONVERSION_FUNCTIONS(ctablegen::DagPair, TableGenDagPairRef);

DEFINE_SIMPLE_CONVERSION_FUNCTIONS(ctablegen::RecordMap::const_iterator,
                                   TableGenRecordKeeperIteratorRef);

DEFINE_SIMPLE_CONVERSION_FUNCTIONS(ArrayRef<SMLoc>, TableGenSourceLocationRef);

#endif
