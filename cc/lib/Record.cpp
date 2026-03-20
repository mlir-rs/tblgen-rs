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
#include <llvm/Config/llvm-config.h>

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

size_t tableGenRecordGetNumTemplateArgs(TableGenRecordRef record_ref) {
  return unwrap(record_ref)->getTemplateArgs().size();
}

TableGenStringRef tableGenRecordGetTemplateArgName(TableGenRecordRef record_ref,
                                                   size_t index) {
  auto args = unwrap(record_ref)->getTemplateArgs();
  if (index >= args.size())
    return TableGenStringRef{.data = nullptr, .len = 0};
  auto name = dyn_cast<StringInit>(args[index]);
  if (!name)
    return TableGenStringRef{.data = nullptr, .len = 0};
  auto val = name->getValue();
  return TableGenStringRef{.data = val.data(), .len = val.size()};
}

size_t tableGenRecordGetNumSuperClasses(TableGenRecordRef record_ref) {
#if LLVM_VERSION_MAJOR >= 21
  return unwrap(record_ref)->getDirectSuperClasses().size();
#else
  return unwrap(record_ref)->getSuperClasses().size();
#endif
}

TableGenRecordRef tableGenRecordGetSuperClass(TableGenRecordRef record_ref,
                                              size_t index) {
#if LLVM_VERSION_MAJOR >= 21
  auto supers = unwrap(record_ref)->getDirectSuperClasses();
#else
  auto supers = unwrap(record_ref)->getSuperClasses();
#endif
  if (index >= supers.size())
    return nullptr;
  return wrap(supers[index].first);
}

TableGenBool tableGenRecordGetValueAsInt(TableGenRecordRef record_ref,
                                         TableGenStringRef name, int64_t *out) {
  auto *rv = unwrap(record_ref)->getValue(StringRef(name.data, name.len));
  if (!rv)
    return false;
  auto *init = dyn_cast<IntInit>(rv->getValue());
  if (!init)
    return false;
  *out = init->getValue();
  return true;
}

TableGenStringRef tableGenRecordGetValueAsString(TableGenRecordRef record_ref,
                                                 TableGenStringRef name) {
  auto *rv = unwrap(record_ref)->getValue(StringRef(name.data, name.len));
  if (!rv)
    return TableGenStringRef{.data = nullptr, .len = 0};
  auto *init = dyn_cast<StringInit>(rv->getValue());
  if (!init)
    return TableGenStringRef{.data = nullptr, .len = 0};
  auto s = init->getValue();
  return TableGenStringRef{.data = s.data(), .len = s.size()};
}

TableGenBool tableGenRecordGetValueAsBit(TableGenRecordRef record_ref,
                                         TableGenStringRef name, int8_t *out) {
  auto *rv = unwrap(record_ref)->getValue(StringRef(name.data, name.len));
  if (!rv)
    return false;
  auto *init = dyn_cast<BitInit>(rv->getValue());
  if (!init)
    return false;
  *out = init->getValue() ? 1 : 0;
  return true;
}

TableGenRecordRef tableGenRecordGetValueAsDef(TableGenRecordRef record_ref,
                                              TableGenStringRef name) {
  auto *rv = unwrap(record_ref)->getValue(StringRef(name.data, name.len));
  if (!rv)
    return nullptr;
  auto *init = dyn_cast<DefInit>(rv->getValue());
  if (!init)
    return nullptr;
  return wrap(const_cast<Record *>(init->getDef()));
}

TableGenTypedInitRef tableGenRecordGetValueAsDag(TableGenRecordRef record_ref,
                                                 TableGenStringRef name) {
  auto *rv = unwrap(record_ref)->getValue(StringRef(name.data, name.len));
  if (!rv)
    return nullptr;
  auto *init = dyn_cast<DagInit>(rv->getValue());
  if (!init)
    return nullptr;
  return wrap(const_cast<TypedInit *>(static_cast<const TypedInit *>(init)));
}

TableGenTypedInitRef
tableGenRecordGetValueAsBitsInit(TableGenRecordRef record_ref,
                                 TableGenStringRef name) {
  auto *rv = unwrap(record_ref)->getValue(StringRef(name.data, name.len));
  if (!rv)
    return nullptr;
  auto *init = dyn_cast<BitsInit>(rv->getValue());
  if (!init)
    return nullptr;
  return wrap(const_cast<TypedInit *>(static_cast<const TypedInit *>(init)));
}

TableGenTypedInitRef
tableGenRecordGetValueAsListInit(TableGenRecordRef record_ref,
                                 TableGenStringRef name) {
  auto *rv = unwrap(record_ref)->getValue(StringRef(name.data, name.len));
  if (!rv)
    return nullptr;
  auto *init = dyn_cast<ListInit>(rv->getValue());
  if (!init)
    return nullptr;
  return wrap(const_cast<TypedInit *>(static_cast<const TypedInit *>(init)));
}

TableGenRecordVectorRef
tableGenRecordGetValueAsListOfDefs(TableGenRecordRef record_ref,
                                   TableGenStringRef name) {
  auto *rv = unwrap(record_ref)->getValue(StringRef(name.data, name.len));
  if (!rv)
    return nullptr;
  auto *list = dyn_cast<ListInit>(rv->getValue());
  if (!list)
    return nullptr;
  auto *vec = new ctablegen::RecordVector();
  vec->reserve(list->size());
  for (size_t i = 0; i < list->size(); ++i) {
    auto *def = dyn_cast<DefInit>(list->getElement(i));
    if (!def) {
      delete vec;
      return nullptr;
    }
    vec->push_back(const_cast<Record *>(def->getDef()));
  }
  return wrap(vec);
}

TableGenBool tableGenRecordGetValueAsListOfInts(TableGenRecordRef record_ref,
                                                TableGenStringRef name,
                                                int64_t **out,
                                                size_t *out_len) {
  auto *rv = unwrap(record_ref)->getValue(StringRef(name.data, name.len));
  if (!rv)
    return false;
  auto *list = dyn_cast<ListInit>(rv->getValue());
  if (!list)
    return false;
  auto n = list->size();
  auto *arr = new int64_t[n];
  for (size_t i = 0; i < n; ++i) {
    auto *elem = dyn_cast<IntInit>(list->getElement(i));
    if (!elem) {
      delete[] arr;
      return false;
    }
    arr[i] = elem->getValue();
  }
  *out = arr;
  *out_len = n;
  return true;
}

TableGenBool tableGenRecordGetValueAsListOfStrings(TableGenRecordRef record_ref,
                                                   TableGenStringRef name,
                                                   TableGenStringRef **out,
                                                   size_t *out_len) {
  auto *rv = unwrap(record_ref)->getValue(StringRef(name.data, name.len));
  if (!rv)
    return false;
  auto *list = dyn_cast<ListInit>(rv->getValue());
  if (!list)
    return false;
  auto n = list->size();
  auto *arr = new TableGenStringRef[n];
  for (size_t i = 0; i < n; ++i) {
    auto *elem = dyn_cast<StringInit>(list->getElement(i));
    if (!elem) {
      delete[] arr;
      return false;
    }
    auto s = elem->getValue();
    arr[i] = TableGenStringRef{.data = s.data(), .len = s.size()};
  }
  *out = arr;
  *out_len = n;
  return true;
}

TableGenBool
tableGenRecordGetValueAsOptionalString(TableGenRecordRef record_ref,
                                       TableGenStringRef name,
                                       TableGenStringRef *out) {
  auto *rv = unwrap(record_ref)->getValue(StringRef(name.data, name.len));
  if (!rv)
    return false;
  if (isa<UnsetInit>(rv->getValue())) {
    *out = TableGenStringRef{.data = nullptr, .len = 0};
    return true;
  }
  auto *init = dyn_cast<StringInit>(rv->getValue());
  if (!init)
    return false;
  auto s = init->getValue();
  *out = TableGenStringRef{.data = s.data(), .len = s.size()};
  return true;
}

TableGenRecordRef
tableGenRecordGetValueAsOptionalDef(TableGenRecordRef record_ref,
                                    TableGenStringRef name) {
  auto *rv = unwrap(record_ref)->getValue(StringRef(name.data, name.len));
  if (!rv)
    return nullptr;
  if (isa<UnsetInit>(rv->getValue()))
    return nullptr;
  auto *init = dyn_cast<DefInit>(rv->getValue());
  if (!init)
    return nullptr;
  return wrap(const_cast<Record *>(init->getDef()));
}

TableGenBool tableGenRecordIsValueUnset(TableGenRecordRef record_ref,
                                        TableGenStringRef name) {
  auto *rv = unwrap(record_ref)->getValue(StringRef(name.data, name.len));
  if (!rv)
    return false;
  return isa<UnsetInit>(rv->getValue());
}

void tableGenIntArrayFree(int64_t *arr) { delete[] arr; }

void tableGenStringRefArrayFree(TableGenStringRef *arr) { delete[] arr; }

TableGenBool tableGenRecordIsClass(TableGenRecordRef record_ref) {
  return unwrap(record_ref)->isClass();
}

TableGenTypedInitRef tableGenRecordGetDefInit(TableGenRecordRef record_ref) {
  return wrap(dyn_cast<TypedInit>(unwrap(record_ref)->getDefInit()));
}

unsigned tableGenRecordGetID(TableGenRecordRef record_ref) {
  return unwrap(record_ref)->getID();
}

TableGenTypedInitRef tableGenRecordGetNameInit(TableGenRecordRef record_ref) {
  return wrap(dyn_cast<TypedInit>(unwrap(record_ref)->getNameInit()));
}

TableGenBool tableGenRecordHasDirectSuperClass(TableGenRecordRef record_ref,
                                               TableGenRecordRef super_ref) {
  return unwrap(record_ref)->hasDirectSuperClass(unwrap(super_ref));
}

size_t tableGenRecordRecTyGetNumClasses(TableGenRecordRef record_ref) {
  return unwrap(record_ref)->getType()->getClasses().size();
}

TableGenRecordRef tableGenRecordRecTyGetClass(TableGenRecordRef record_ref,
                                              size_t index) {
  auto classes = unwrap(record_ref)->getType()->getClasses();
  if (index >= classes.size())
    return nullptr;
  return wrap(const_cast<Record *>(classes[index]));
}

TableGenBool tableGenRecordRecTyIsSubClassOf(TableGenRecordRef record_ref,
                                             TableGenRecordRef class_ref) {
  return unwrap(record_ref)->getType()->isSubClassOf(unwrap(class_ref));
}
