// Original work Copyright 2016 Alexander Stocko <as@coder.gg>.
// Modified work Copyright 2023 Daan Vanoverloop
// See the COPYRIGHT file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#include "TableGen.hpp"
#include "Types.h"

using namespace llvm;
using ctablegen::RecordMap;

void tableGenRecordKeeperFree(TableGenRecordKeeperRef rk_ref) {
  delete unwrap(rk_ref);
}

TableGenRecordKeeperIteratorRef
tableGenRecordKeeperGetFirstClass(TableGenRecordKeeperRef rk_ref) {
  auto &classes = unwrap(rk_ref)->getClasses();
  if (classes.begin() == classes.end())
    return nullptr;
  return wrap(new ctablegen::RecordMapIterator{classes.begin(), classes.end()});
}

TableGenRecordKeeperIteratorRef
tableGenRecordKeeperGetFirstDef(TableGenRecordKeeperRef rk_ref) {
  auto &defs = unwrap(rk_ref)->getDefs();
  if (defs.begin() == defs.end())
    return nullptr;
  return wrap(new ctablegen::RecordMapIterator{defs.begin(), defs.end()});
}

void tableGenRecordKeeperGetNextClass(TableGenRecordKeeperIteratorRef *item) {
  auto *iter = unwrap(*item);
  if (++iter->it == iter->end) {
    delete iter;
    *item = nullptr;
  }
}

void tableGenRecordKeeperGetNextDef(TableGenRecordKeeperIteratorRef *item) {
  auto *iter = unwrap(*item);
  if (++iter->it == iter->end) {
    delete iter;
    *item = nullptr;
  }
}

void tableGenRecordKeeperIteratorFree(TableGenRecordKeeperIteratorRef item) {
  if (item)
    delete unwrap(item);
}

TableGenRecordKeeperIteratorRef
tableGenRecordKeeperIteratorClone(TableGenRecordKeeperIteratorRef item) {
  auto *iter = unwrap(item);
  return wrap(new ctablegen::RecordMapIterator{iter->it, iter->end});
}

TableGenStringRef
tableGenRecordKeeperItemGetName(TableGenRecordKeeperIteratorRef item) {
  auto &s = unwrap(item)->it->first;
  return TableGenStringRef{.data = s.data(), .len = s.size()};
}

TableGenRecordRef
tableGenRecordKeeperItemGetRecord(TableGenRecordKeeperIteratorRef item) {
  return wrap(unwrap(item)->it->second.get());
}

TableGenRecordMapRef
tableGenRecordKeeperGetClasses(TableGenRecordKeeperRef rk_ref) {
  return wrap(&unwrap(rk_ref)->getClasses());
}

TableGenRecordMapRef
tableGenRecordKeeperGetDefs(TableGenRecordKeeperRef rk_ref) {
  return wrap(&unwrap(rk_ref)->getDefs());
}

TableGenRecordRef tableGenRecordKeeperGetClass(TableGenRecordKeeperRef rk_ref,
                                               TableGenStringRef name) {
  return wrap(unwrap(rk_ref)->getClass(StringRef(name.data, name.len)));
}

TableGenRecordRef tableGenRecordKeeperGetDef(TableGenRecordKeeperRef rk_ref,
                                             TableGenStringRef name) {
  return wrap(unwrap(rk_ref)->getDef(StringRef(name.data, name.len)));
}

TableGenRecordVectorRef
tableGenRecordKeeperGetAllDerivedDefinitions(TableGenRecordKeeperRef rk_ref,
                                             TableGenStringRef className) {
  return wrap(
      new ctablegen::RecordVector(unwrap(rk_ref)->getAllDerivedDefinitions(
          StringRef(className.data, className.len))));
}

TableGenRecordRef tableGenRecordVectorGet(TableGenRecordVectorRef vec_ref,
                                          size_t index) {
  auto *vec = unwrap(vec_ref);
  if (index < vec->size())
    return wrap(((*vec)[index]));
  return nullptr;
}

size_t tableGenRecordVectorSize(TableGenRecordVectorRef vec_ref) {
  return unwrap(vec_ref)->size();
}

void tableGenRecordVectorFree(TableGenRecordVectorRef vec_ref) {
  delete unwrap(vec_ref);
}

TableGenRecordVectorRef tableGenRecordKeeperGetAllDerivedDefinitionsIfDefined(
    TableGenRecordKeeperRef rk_ref, TableGenStringRef className) {
  return wrap(new ctablegen::RecordVector(
      unwrap(rk_ref)->getAllDerivedDefinitionsIfDefined(
          StringRef(className.data, className.len))));
}

TableGenStringRef
tableGenRecordKeeperGetInputFilename(TableGenRecordKeeperRef rk_ref) {
  auto name = unwrap(rk_ref)->getInputFilename();
  return TableGenStringRef{.data = name.data(), .len = name.size()};
}

TableGenTypedInitRef
tableGenRecordKeeperGetGlobal(TableGenRecordKeeperRef rk_ref,
                              TableGenStringRef name) {
  auto *init = unwrap(rk_ref)->getGlobal(StringRef(name.data, name.len));
  if (!init)
    return nullptr;
  return wrap(dyn_cast<TypedInit>(init));
}
