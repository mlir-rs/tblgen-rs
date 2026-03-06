// Original work Copyright 2016 Alexander Stocko <as@coder.gg>.
// Modified work Copyright 2023 Daan Vanoverloop
// See the COPYRIGHT file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#ifndef _CTABLEGEN_TABLEGEN_H_
#define _CTABLEGEN_TABLEGEN_H_

#ifdef __cplusplus
#include <cstddef>
#include <cstdint>
#else
#include <stddef.h>
#include <stdint.h>
#endif

#include "Types.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef enum {
  TABLEGEN_DK_ERROR,
  TABLEGEN_DK_WARNING,
  TABLEGEN_DK_REMARK,
  TABLEGEN_DK_NOTE,
} TableGenDiagKind;

typedef enum {
  TableGenBitRecTyKind,
  TableGenBitsRecTyKind,
  TableGenCodeRecTyKind,
  TableGenIntRecTyKind,
  TableGenStringRecTyKind,
  TableGenListRecTyKind,
  TableGenDagRecTyKind,
  TableGenRecordRecTyKind,
  TableGenInvalidRecTyKind
} TableGenRecTyKind;

typedef struct TableGenStringRef {
  const char *data;
  size_t len;
} TableGenStringRef;

typedef void (*TableGenStringCallback)(TableGenStringRef, void *);

TableGenParserRef tableGenGet();
void tableGenFree(TableGenParserRef tg_ref);
TableGenBool tableGenAddSource(TableGenParserRef tg_ref, const char *source);
void tableGenAddSourceFile(TableGenParserRef tg_ref, TableGenStringRef source);
void tableGenAddIncludeDirectory(TableGenParserRef tg_ref,
                                 TableGenStringRef include);

/// NOTE: TableGen currently relies on global state within a given parser
///       invocation, so this function is not thread-safe.
TableGenRecordKeeperRef tableGenParse(TableGenParserRef tg_ref);

// LLVM RecordKeeper
void tableGenRecordKeeperFree(TableGenRecordKeeperRef rk_ref);
TableGenRecordMapRef
tableGenRecordKeeperGetClasses(TableGenRecordKeeperRef rk_ref);
TableGenRecordMapRef
tableGenRecordKeeperGetDefs(TableGenRecordKeeperRef rk_ref);
TableGenRecordRef tableGenRecordKeeperGetClass(TableGenRecordKeeperRef rk_ref,
                                               TableGenStringRef name);
TableGenRecordRef tableGenRecordKeeperGetDef(TableGenRecordKeeperRef rk_ref,
                                             TableGenStringRef name);
TableGenRecordVectorRef
tableGenRecordKeeperGetAllDerivedDefinitions(TableGenRecordKeeperRef rk_ref,
                                             TableGenStringRef className);

TableGenRecordRef tableGenRecordVectorGet(TableGenRecordVectorRef vec_ref,
                                          size_t index);
void tableGenRecordVectorFree(TableGenRecordVectorRef vec_ref);

TableGenRecordKeeperIteratorRef
tableGenRecordKeeperGetFirstClass(TableGenRecordKeeperRef rk_ref);

TableGenRecordKeeperIteratorRef
tableGenRecordKeeperGetFirstDef(TableGenRecordKeeperRef rk_ref);

void tableGenRecordKeeperGetNextClass(TableGenRecordKeeperIteratorRef *item);
void tableGenRecordKeeperGetNextDef(TableGenRecordKeeperIteratorRef *item);

TableGenStringRef
tableGenRecordKeeperItemGetName(TableGenRecordKeeperIteratorRef item);
TableGenRecordRef
tableGenRecordKeeperItemGetRecord(TableGenRecordKeeperIteratorRef item);
void tableGenRecordKeeperIteratorFree(TableGenRecordKeeperIteratorRef item);
TableGenRecordKeeperIteratorRef
tableGenRecordKeeperIteratorClone(TableGenRecordKeeperIteratorRef item);

// LLVM Record
TableGenRecordKeeperRef tableGenRecordGetRecords(TableGenRecordRef record_ref);
TableGenStringRef tableGenRecordGetName(TableGenRecordRef record_ref);
TableGenRecordValRef tableGenRecordGetValue(TableGenRecordRef record_ref,
                                            TableGenStringRef name);
TableGenRecTyKind tableGenRecordGetFieldType(TableGenRecordRef record_ref,
                                             TableGenStringRef name);
TableGenBool tableGenRecordIsAnonymous(TableGenRecordRef record_ref);
TableGenBool tableGenRecordIsSubclassOf(TableGenRecordRef record_ref,
                                        TableGenStringRef name);
void tableGenRecordPrint(TableGenRecordRef record_ref,
                         TableGenStringCallback callback, void *userData);
void tableGenRecordDump(TableGenRecordRef record_ref);
TableGenSourceLocationRef tableGenRecordGetLoc(TableGenRecordRef record_ref);

// LLVM RecordVal
TableGenStringRef tableGenRecordValGetName(TableGenRecordValRef rv_ref);
TableGenTypedInitRef tableGenRecordValGetNameInit(TableGenRecordValRef rv_ref);
TableGenRecTyKind tableGenRecordValGetType(TableGenRecordValRef rv_ref);
TableGenTypedInitRef tableGenRecordValGetValue(TableGenRecordValRef rv_ref);
void tableGenRecordValTest(TableGenRecordValRef rv_ref);
TableGenRecordValRef tableGenRecordGetFirstValue(TableGenRecordRef record_ref);
TableGenRecordValRef tableGenRecordValNext(TableGenRecordRef record,
                                           TableGenRecordValRef current);
void tableGenRecordValPrint(TableGenRecordValRef rv_ref,
                            TableGenStringCallback callback, void *userData);
void tableGenRecordValDump(TableGenRecordValRef rv_ref);
TableGenSourceLocationRef tableGenRecordValGetLoc(TableGenRecordValRef rv_ref);

char *tableGenRecordValGetValAsNewString(TableGenRecordValRef rv_ref);
TableGenBool tableGenRecordValGetValAsBit(TableGenRecordValRef rv_ref,
                                          int8_t *bit);
int8_t *tableGenRecordValGetValAsBits(TableGenRecordValRef rv_ref, size_t *len);
TableGenBool tableGenRecordValGetValAsInt(TableGenRecordValRef rv_ref,
                                          int64_t *integer);
TableGenRecordRef tableGenRecordValGetValAsRecord(TableGenRecordValRef rv_ref);
TableGenRecordRef
tableGenRecordValGetValAsDefRecord(TableGenRecordValRef rv_ref);

// LLVM ListType
TableGenRecTyKind tableGenListRecordGetType(TableGenRecordValRef rv_ref);
TableGenTypedInitRef tableGenListRecordGet(TableGenTypedInitRef rv_ref,
                                           size_t index);
size_t tableGenListRecordNumElements(TableGenTypedInitRef rv_ref);

// LLVM DagType
TableGenRecordRef tableGenDagRecordOperator(TableGenTypedInitRef rv_ref);
TableGenTypedInitRef tableGenDagRecordGet(TableGenTypedInitRef rv_ref,
                                          size_t index);
TableGenStringRef tableGenDagRecordArgName(TableGenTypedInitRef rv_ref,
                                           size_t index);
size_t tableGenDagRecordNumArgs(TableGenTypedInitRef rv_ref);

// Utility
TableGenRecTyKind tableGenInitRecType(TableGenTypedInitRef ti);
TableGenBool tableGenBitInitGetValue(TableGenTypedInitRef ti, int8_t *bit);
int8_t *tableGenBitsInitGetValue(TableGenTypedInitRef ti, size_t *len);
TableGenBool tableGenBitsInitGetNumBits(TableGenTypedInitRef ti, size_t *len);
TableGenTypedInitRef tableGenBitsInitGetBitInit(TableGenTypedInitRef ti,
                                                size_t index);
TableGenBool tableGenIntInitGetValue(TableGenTypedInitRef ti, int64_t *integer);
TableGenStringRef tableGenStringInitGetValue(TableGenTypedInitRef ti);
char *tableGenStringInitGetValueNewString(TableGenTypedInitRef ti);
TableGenRecordRef tableGenDefInitGetValue(TableGenTypedInitRef ti);
void tableGenInitPrint(TableGenTypedInitRef ti, TableGenStringCallback callback,
                       void *userData);
void tableGenInitDump(TableGenTypedInitRef ti);
TableGenBool tableGenPrintError(TableGenParserRef ref,
                                TableGenSourceLocationRef loc_ref,
                                TableGenDiagKind dk, TableGenStringRef message,
                                TableGenStringCallback callback,
                                void *userData);
TableGenSourceLocationRef tableGenSourceLocationNull();
TableGenSourceLocationRef
tableGenSourceLocationClone(TableGenSourceLocationRef loc_ref);

// Memory
void tableGenSourceLocationFree(TableGenSourceLocationRef loc_ref);
void tableGenBitArrayFree(int8_t bit_array[]);
void tableGenStringFree(const char *str);
void tableGenStringArrayFree(const char **str_array);

#ifdef __cplusplus
}
#endif
#endif
