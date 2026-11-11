# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres
to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

## [0.2.2] - 2026-11-11

### Added

- `RotationWidget::set_natural_child_size_enabled` /
  `with_natural_child_size_enabled` / `natural_child_size_enabled` -
  opt-in for allocating the child its measured natural size. Default
  `false` keeps the legacy inverse-solve allocation, which is
  ill-conditioned near 45° (and symmetric bands) and shrank non-square
  children into a smaller square. Enabled, the child keeps its natural
  size; `measure` already reports the rotated bounding box.
- `examples/interactive_demo.rs` and `examples/nested_demo.rs` gained a
  "Natural Child Size" switch to compare both allocation modes live.

## [0.2.1] - 2026-11-09

### Added

- `RotationWidget::set_input_region_enabled` /
  `with_input_region_enabled` / `input_region_enabled` - opt-out for
  the surface input-region clamp (`size_allocate` used to apply it
  unconditionally). Default `true` keeps the layer-shell behavior;
  nested widgets disable it so the clamp does not kill input for the
  rest of the window.

### Changed

### Fixed

- The input region is now translated into surface coordinates -
  previously the region was built in widget space (`0..w x 0..h`),
  which only matched when the widget filled the whole surface.
- `unmap` resets the surface input region - a removed or hidden
  `RotationWidget` no longer leaves the surface input-clamped.
- Disabling the input region at runtime (`set_input_region_enabled`)
  immediately resets a previously applied clamp.

### Distribution

### Infrastructure

---

## [0.2.0] - 2026-08-15

### Added

- Book
- Rustdoc

### Changed

- Make layer shell dependency optional (enabled by default)
- Make rotation gesture optional (enabled by default)

### Infrastructure

- Updated github action workflows
- Fixed MSRV check in CI
- Publish book and docs to gh-pages
- Publish crate to crates.io (CI)

