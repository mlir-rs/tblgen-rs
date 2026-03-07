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
using ctablegen::tableGenFromRecType;

TableGenRecordKeeperRef tableGenRecordGetRecords(TableGenRecordRef record_ref) {
  return wrap(&unwrap(record_ref)->getRecords());
}

TableGenStringRef tableGenRecordGetName(TableGenRecordRef record_ref) {
  auto name = unwrap(record_ref)->getName();
  return TableGenStringRef{.data = name.data(), .len = name.size()};
}

TableGenRecordValRef tableGenRecordGetValue(TableGenRecordRef record_ref,
                                            TableGenStringRef name) {
  return wrap(unwrap(record_ref)->getValue(StringRef(name.data, name.len)));
}

TableGenRecTyKind tableGenRecordGetFieldType(TableGenRecordRef record_ref,
                                             TableGenStringRef name) {
  auto value = unwrap(record_ref)->getValue(StringRef(name.data, name.len));
  if (!value)
    return TableGenInvalidRecTyKind;
  return tableGenFromRecType(value->getType());
}

TableGenRecordValRef tableGenRecordGetFirstValue(TableGenRecordRef record_ref) {
  auto values = unwrap(record_ref)->getValues();
  if (values.empty()) {
    return nullptr;
  }
  return wrap(values.begin());
}

TableGenRecordValRef tableGenRecordValNext(TableGenRecordRef record,
                                           TableGenRecordValRef current) {
  auto next = std::next(ArrayRef<RecordVal>::iterator(unwrap(current)));
  if (next == unwrap(record)->getValues().end()) {
    return nullptr;
  }
  return wrap(next);
}

TableGenBool tableGenRecordIsAnonymous(TableGenRecordRef record_ref) {
  return unwrap(record_ref)->isAnonymous();
}

TableGenBool tableGenRecordIsSubclassOf(TableGenRecordRef record_ref,
                                        TableGenStringRef name) {
  return unwrap(record_ref)->isSubClassOf(StringRef(name.data, name.len));
}

TableGenSourceLocationRef tableGenRecordGetLoc(TableGenRecordRef record_ref) {
  return wrap(new ArrayRef(unwrap(record_ref)->getLoc()));
}

void tableGenRecordPrint(TableGenRecordRef record_ref,
                         TableGenStringCallback callback, void *userData) {
  ctablegen::CallbackOstream stream(callback, userData);
  stream << *unwrap(record_ref);
}

void tableGenRecordDump(TableGenRecordRef record_ref) {
  unwrap(record_ref)->dump();
}
