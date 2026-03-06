# Changelog

## [0.7.3] - Unreleased

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
