# Changelog

## [0.9.0] - 2026-03-20

### Added

- LLVM 22 support (#59)
- `BitsInit::known_bits_to_int()` (LLVM 22 only)
- Direct typed value accessors on `Record`: `int_value`, `str_value`, `bit_value`, `def_value`, `dag_value`, `bits_init_value`, `list_init_value`, `list_of_defs_value`, `list_of_ints_value`, `list_of_strings_value`, `optional_str_value`, `optional_def_value`, `is_value_unset`
- Record identity/metadata: `is_class`, `def_init`, `id`, `name_init`, `has_direct_super_class`
- RecordRecTy accessors: `num_type_classes`, `type_class`, `type_is_subclass_of`
- RecordValue metadata: `is_template_arg`, `is_nonconcrete_ok`, `bits_width`, `list_element_type`
- `DagInit::arg_no` for named argument lookup
- `RecordKeeper::input_filename` and `RecordKeeper::global`
- LLVM 21 test matrix in CI

### Changed

- Default LLVM version is now 22

## [0.8.0] - Unreleased

### Fixed

- Fix soundness bugs and iterator memory leaks (#50)
  - `tableGenRecordGetFirstValue` now returns `nullptr` for empty records, preventing UB when iterating a record with no fields
  - `RecordValueIter::next()` no longer calls `tableGenRecordValNext` with a null pointer after the iterator is exhausted
  - `NamedRecordIter::clone()` no longer segfaults when called on an exhausted iterator
  - `GetNextClass`/`GetNextDef` now correctly free the heap-allocated iterator object when the end is reached
  - Memory leak in `TableGenParser::parse()` on failed file inclusion fixed via `std::unique_ptr`
  - Replace panicking `From<BitInit> for bool`, `From<BitsInit> for Vec<bool>`, and `From<IntInit> for i64` with `TryFrom` impls that return errors instead of panicking on variable references or C API failures
- Strip `-D_FORTIFY_SOURCE` from llvm-config flags in `build.rs` (#49)
- Fix `DagIter` stopping early on unnamed dag arguments (#48)
- Add `From<BitsInit> for Vec<Option<bool>>` (#47)
- Fix UB in `BitsInit::bit()` when bit is a `VarBitInit` (#46)
- Fix llvm-config static link detection and exit status handling (#45)

### Added

- `RecordIter` now implements `ExactSizeIterator` and `size_hint()` (#50)

## [0.7.2] - 2025-11-04

### Fixed

- Revert invalid-to-string conversion (#41)

## [0.7.1] - 2025-10-14

### Added

- Conversion from `Invalid` init to `String` (#40)

### Fixed

- Map `code` type correctly (#38)

## [0.7.0] - 2025-09-25

### Added

- LLVM 21 support (#31)
