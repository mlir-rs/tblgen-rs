// Original work Copyright 2016 Alexander Stocko <as@coder.gg>.
// Modified work Copyright 2023 Daan Vanoverloop
// See the COPYRIGHT file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#include "TableGen.h"
#include "TableGen.hpp"
#include "Types.h"
#include "llvm/Support/SourceMgr.h"

using namespace llvm;

namespace ctablegen {

TableGenRecTyKind tableGenFromRecType(const RecTy *rt) {
  switch (rt->getRecTyKind()) {
  case RecTy::BitRecTyKind:
    return TableGenBitRecTyKind;
  case RecTy::BitsRecTyKind:
    return TableGenBitsRecTyKind;
  case RecTy::IntRecTyKind:
    return TableGenIntRecTyKind;
  case RecTy::StringRecTyKind:
    return TableGenStringRecTyKind;
  case RecTy::ListRecTyKind:
    return TableGenListRecTyKind;
  case RecTy::DagRecTyKind:
    return TableGenDagRecTyKind;
  case RecTy::RecordRecTyKind:
    return TableGenRecordRecTyKind;
  default:
    return TableGenInvalidRecTyKind;
  }
}

} // namespace ctablegen

TableGenRecTyKind tableGenInitRecType(TableGenTypedInitRef ti) {
  if (!ti)
    return TableGenInvalidRecTyKind;
  auto typed_init = dyn_cast<TypedInit>(unwrap(ti));
  if (!typed_init)
    return TableGenInvalidRecTyKind;
  return ctablegen::tableGenFromRecType(typed_init->getType());
}

TableGenBool tableGenBitInitGetValue(TableGenTypedInitRef ti, int8_t *bit) {
  if (!ti)
    return false;
  auto bit_init = dyn_cast<BitInit>(unwrap(ti));
  if (!bit_init)
    return -1;
  *bit = bit_init->getValue();
  return true;
}

TableGenBool tableGenBitsInitGetNumBits(TableGenTypedInitRef ti, size_t *len) {
  if (!ti)
    return false;
  auto bits_init = dyn_cast<BitsInit>(unwrap(ti));
  if (!bits_init)
    return false;

  *len = bits_init->getNumBits();
  return true;
}

TableGenTypedInitRef tableGenBitsInitGetBitInit(TableGenTypedInitRef ti,
                                                size_t index) {
  if (!ti)
    return nullptr;
  auto bits_init = dyn_cast<BitsInit>(unwrap(ti));
  if (!bits_init)
    return nullptr;

  // Return the raw Init* -- may be BitInit or VarBitInit.
  // Caller must use tableGenBitInitIsVarBit() to distinguish.
  return wrap(
      const_cast<TypedInit *>(dyn_cast<TypedInit>(bits_init->getBit(index))));
}

TableGenBool tableGenIntInitGetValue(TableGenTypedInitRef ti,
                                     int64_t *integer) {
  if (!ti)
    return false;
  auto int_init = dyn_cast<IntInit>(unwrap(ti));
  if (!int_init)
    return false;

  *integer = int_init->getValue();
  return true;
}

TableGenStringRef tableGenStringInitGetValue(TableGenTypedInitRef ti) {
  if (!ti)
    return TableGenStringRef{.data = nullptr, .len = 0};
  auto str_init = dyn_cast<StringInit>(unwrap(ti));
  if (!str_init)
    return TableGenStringRef{.data = nullptr, .len = 0};
  auto val = str_init->getValue();
  return TableGenStringRef{.data = val.data(), .len = val.size()};
}

char *tableGenStringInitGetValueNewString(TableGenTypedInitRef ti) {
  if (!ti)
    return nullptr;
  auto str_init = dyn_cast<StringInit>(unwrap(ti));
  if (!str_init)
    return nullptr;

  auto val = str_init->getValue();
  auto sz = val.size();
  auto str = new char[sz + 1];
  std::copy(val.begin(), val.end(), str);
  str[sz] = '\0';
  return str;
}

TableGenRecordRef tableGenDefInitGetValue(TableGenTypedInitRef ti) {
  if (!ti)
    return nullptr;
  auto def_init = dyn_cast<DefInit>(unwrap(ti));
  if (!def_init)
    return nullptr;
  return wrap(def_init->getDef());
}

void tableGenInitPrint(TableGenTypedInitRef ti, TableGenStringCallback callback,
                       void *userData) {
  ctablegen::CallbackOstream stream(callback, userData);
  stream << *unwrap(ti);
}

void tableGenInitDump(TableGenTypedInitRef ti) { unwrap(ti)->dump(); }

// VarBitInit support: exposes LLVM's VarBitInit for variable bit references
// (e.g., lda{17}) that BitsInit::getBit() may return instead of BitInit.

TableGenBool tableGenBitInitIsVarBit(TableGenTypedInitRef ti) {
  if (!ti)
    return false;
  return isa<VarBitInit>(unwrap(ti));
}

TableGenStringRef tableGenVarBitInitGetVarName(TableGenTypedInitRef ti) {
  if (!ti)
    return TableGenStringRef{.data = nullptr, .len = 0};
  auto var_bit = dyn_cast<VarBitInit>(unwrap(ti));
  if (!var_bit)
    return TableGenStringRef{.data = nullptr, .len = 0};
  auto var_init = dyn_cast<VarInit>(var_bit->getBitVar());
  if (!var_init)
    return TableGenStringRef{.data = nullptr, .len = 0};
  auto name = var_init->getName();
  return TableGenStringRef{.data = name.data(), .len = name.size()};
}

size_t tableGenVarBitInitGetBitNum(TableGenTypedInitRef ti) {
  if (!ti)
    return 0;
  auto var_bit = dyn_cast<VarBitInit>(unwrap(ti));
  if (!var_bit)
    return 0;
  return var_bit->getBitNum();
}

TableGenBool tableGenPrintError(TableGenParserRef ref,
                                TableGenSourceLocationRef loc_ref,
                                TableGenDiagKind dk, TableGenStringRef message,
                                TableGenStringCallback callback,
                                void *userData) {
  ctablegen::CallbackOstream stream(callback, userData);
  ArrayRef<SMLoc> Loc = *unwrap(loc_ref);

  SMLoc NullLoc;
  if (Loc.empty())
    Loc = NullLoc;
  auto &SrcMgr = unwrap(ref)->sourceMgr;

  if (!SrcMgr.FindBufferContainingLoc(Loc.front()))
    return false;

  SrcMgr.PrintMessage(stream, Loc.front(), static_cast<SourceMgr::DiagKind>(dk),
                      StringRef(message.data, message.len));

  for (unsigned i = 1; i < Loc.size(); ++i) {
    if (!SrcMgr.FindBufferContainingLoc(Loc[i]))
      continue;
    SrcMgr.PrintMessage(stream, Loc[i], llvm::SourceMgr::DK_Note,
                        "initiated from multiclass");
  }

  return true;
}

TableGenSourceLocationRef tableGenSourceLocationNull() {
  auto source_loc = SMLoc();
  return wrap(new ArrayRef(source_loc));
}

TableGenSourceLocationRef
tableGenSourceLocationClone(TableGenSourceLocationRef loc_ref) {
  return wrap(new ArrayRef(*unwrap(loc_ref)));
}

void tableGenSourceLocationFree(TableGenSourceLocationRef loc_ref) {
  delete unwrap(loc_ref);
}
