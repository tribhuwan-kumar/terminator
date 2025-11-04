# Changelog

All notable changes to Terminator will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.22.5] - 2025-11-04

### Fixed
- Chain selector parsing with outer parentheses - selectors like `(role:Window && name:Calculator) >> (role:Custom && nativeid:NavView)` now parse correctly at runtime

### Changed
- Separated selector tests into dedicated `selector_tests.rs` file for better code organization
- Reduced `selector.rs` from 1,129 to 621 lines (implementation only)

## [0.22.2] - 2025-11-03

### Added
- Debug tests for selector functionality

### Fixed
- Cleanup of problematic selector tests

### Changed
- Updated issue creation link in skill.md

## [0.22.1] - 2025-11-03

### Fixed
- Boolean selectors handling
- UIA element capture timeout increased to 5s with automatic fallback

### Changed
- Workflow recorder timeout improvements

## [0.20.6] - 2025-10-31

### Added
- Initial CHANGELOG.md
- Release command for automated version management
