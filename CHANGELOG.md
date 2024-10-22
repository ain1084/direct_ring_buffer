# Changelog

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
