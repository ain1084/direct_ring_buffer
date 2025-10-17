# Changelog

## [0.2.2] - 2025-10-17

### Added

- New unit test `test_empty_write`.
- Implemented `Drop` for safe memory deallocation.
- Expanded README examples:
  - Minimal slice-based example.
  - Wrap-around example.
  - Single-element example for intuitive understanding.

### Changed

- Internal buffer initialization now uses `Vec::resize_with`, ensuring proper element construction with `T::default()`.
- `Copy` trait bound relaxed — initialization safety now relies on `Default`.
- Improved Miri compatibility: the library now passes all Miri checks without undefined behavior (UB).
- Reorganized README structure (moved **Features** before **Overview**) for better readability.

### Fixed

- Corrected documentation indentation and formatting.
- Updated dependencies to the latest compatible versions.

### Improved

- Clarified trait requirements and `Drop` behavior in documentation.
- Refined Criterion benchmarks for `write_element` / `read_element`:
  - Fixed unrealistic timing caused by buffer reuse.
  - Benchmarks now reinitialize buffers per iteration for accurate throughput measurement.

## [0.2.1] - 2024-10-23

### Added

- Introduced .devcontainer for development in a containerized environment, enabling easier setup with VS Code and Docker.

### Improved

- Simplified the example in README.md to better demonstrate slice-based operations and wrap-around behavior.

## [0.2.0] - 2024-06-03

### Added

- Introduced `write_slices` method in `Producer` for writing elements using slices.
- Introduced `read_slices` method in `Consumer` for reading elements using slices.
- Deprecated the `write` method in `Producer` in favor of `write_slices`.
- Deprecated the `read` method in `Consumer` in favor of `read_slices`.

### Changed

- Updated documentation to reflect the new methods (`write_slices` and `read_slices`).
- Modified example code in the documentation to use `write_slices` and `read_slices` 
  instead of the deprecated `write` and `read`.
- Example code now includes usage of `write_element` and `read_element`.

### Fixed

- Improved documentation and comments to provide clearer guidance and updated method names.
