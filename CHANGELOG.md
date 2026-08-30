# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.6.2](https://github.com/Isvane/fuzzies/compare/fuzzies-v0.6.1...fuzzies-v0.6.2) - 2026-08-30

### Documentation

- update README description
- link Node README.md

### Other

- 1.0.0 ([#71](https://github.com/Isvane/fuzzies/pull/71))

## [0.6.1](https://github.com/Isvane/fuzzies/compare/v0.6.0...v0.6.1) - 2026-08-26

### Added

- *(dictionary)* add `suggest` method ([#68](https://github.com/Isvane/fuzzies/pull/68))
- feat(SearchResult) add `similarity` function ([#67](https://github.com/Isvane/fuzzies/pull/67))

### Documentation

- update README.md
- update benchmark data
- document safety

### Refactored

- *(dictionary)* store `DictionarySource` directly ([#66](https://github.com/Isvane/fuzzies/pull/66))

## [0.6.0](https://github.com/Isvane/fuzzies/compare/v0.5.4...v0.6.0) - 2026-08-19

### Added

- *(dictionary)* add `from_iterator` method ([#64](https://github.com/Isvane/fuzzies/pull/64))

### Documentation

- update benchmark data

### Refactored

- *(search)* unify search builder ([#63](https://github.com/Isvane/fuzzies/pull/63))
- *(search)* [**breaking**] replace `is_exact` field with helper method ([#62](https://github.com/Isvane/fuzzies/pull/62))

## [0.5.4](https://github.com/Isvane/fuzzies/compare/v0.5.3...v0.5.4) - 2026-07-28

### Documentation

- *(examples)* add demo.rs

### Other

- *(bench)* expand benchmark coverage
- *(search)* early return candidates exceeding worst distance bound

## [0.5.3](https://github.com/Isvane/fuzzies/compare/v0.5.2...v0.5.3) - 2026-07-26

### Added

- *(search)* add range bound filters ([#59](https://github.com/Isvane/fuzzies/pull/59))
- *(dictionary)* add `len` and `is_empty` methods

### Documentation

- update README.md

## [0.5.2](https://github.com/Isvane/fuzzies/compare/v0.5.1...v0.5.2) - 2026-07-19

### Documentation

- fix docs error

### Fixed

- *(search)* transposition priority over alphabetical ties

## [0.5.1](https://github.com/Isvane/fuzzies/compare/v0.5.0...v0.5.1) - 2026-07-19

### Added

- *(search)* implement Display for SearchResult

## [0.5.0](https://github.com/Isvane/fuzzies/compare/v0.4.3...v0.5.0) - 2026-07-18

### Added

- *(search)* [**breaking**] refactor batch_search to use builder pattern ([#55](https://github.com/Isvane/fuzzies/pull/55))

### Other

- update .release-plz.toml

### Refactored

- [**breaking**] encapsulate internal FST types and streamline docstrings

## [0.4.3](https://github.com/Isvane/fuzzies/compare/v0.4.2...v0.4.3) - 2026-07-17

### Added

- *(search)* implement Ord and PartialOrd for SearchResult

### Documentation

- *(examples)* add dictionary search example
- improve README ([#53](https://github.com/Isvane/fuzzies/pull/53))
- remove motivation ([#52](https://github.com/Isvane/fuzzies/pull/52))
- update README.md ([#50](https://github.com/Isvane/fuzzies/pull/50))

## [0.4.2](https://github.com/Isvane/fuzzies/compare/v0.4.1...v0.4.2) - 2026-07-06

### Added

- *(dictionary)* add contains method ([#48](https://github.com/Isvane/fuzzies/pull/48))
- *(dictionary)* add support for embedded FST ([#45](https://github.com/Isvane/fuzzies/pull/45))

### Documentation

- update README with improved usage details ([#49](https://github.com/Isvane/fuzzies/pull/49))
- *(examples)* add interactive repl fuzzy search example ([#44](https://github.com/Isvane/fuzzies/pull/44))
- revise README.md ([#39](https://github.com/Isvane/fuzzies/pull/39))

## [0.4.1](https://github.com/Isvane/fuzzies/compare/v0.4.0...v0.4.1) - 2026-07-03

### Added

- *(search)* add prefix fuzzy search support ([#37](https://github.com/Isvane/fuzzies/pull/37))

## [0.4.0](https://github.com/Isvane/fuzzies/compare/v0.3.0...v0.4.0) - 2026-06-28

### Added

- *(search)* [**breaking**] support transposition ([#33](https://github.com/Isvane/fuzzies/pull/33))

### Continuous Integration

- optimize Rust workflow and configure release-plz changelog/releases ([#30](https://github.com/Isvane/fuzzies/pull/30))

### Documentation

- update README.md ([#34](https://github.com/Isvane/fuzzies/pull/34))
- add performance benchmarks ([#31 ](https://github.com/Isvane/fuzzies/pull/31))
- streamline dictionary API documentation and add examples ([#28](https://github.com/Isvane/fuzzies/pull/28))

### Refactored

- replace Box<dyn Error> with thiserror custom enum ([e73f](e73f469371c743478f48c7b056cd044ddeadac4f))
- *(search)* [**breaking**] cap max Levenshtein distance at 2 ([#32](https://github.com/Isvane/fuzzies/pull/32))
</blockquote>


## [0.3.0](https://github.com/Isvane/fuzzies/compare/v0.2.1...v0.3.0) - 2026-06-25

### Fixed

- *(search)* [**breaking**] fix sorting by distance bug ([#25](https://github.com/Isvane/fuzzies/pull/25))
- *(search)* prioritize Levenshtein distance over alphabetical order ([#22](https://github.com/Isvane/fuzzies/pull/22))

### Other

- add example for dictionary method ([#26](https://github.com/Isvane/fuzzies/pull/26))
- remove redundant logic ([#27](https://github.com/Isvane/fuzzies/pull/27)

## [0.2.1](https://github.com/Isvane/fuzzies/compare/v0.2.0...v0.2.1) - 2026-06-24

### Added

- *(dictionary)* add in-place text file sorting helper and tests ([#18](https://github.com/Isvane/fuzzies/pull/18))

### Other

- add GitHub Actions CI workflow for Rust ([#17](https://github.com/Isvane/fuzzies/pull/17))
- *(benches)* update benchmark ([#16](https://github.com/Isvane/fuzzies/pull/16))
- add can_match to prune dead branches ([#14](https://github.com/Isvane/fuzzies/pull/14))

## [0.2.0](https://github.com/Isvane/fuzzies/compare/v0.1.0...v0.2.0) - 2026-06-23

### Added

- *(search)* support configurable Levenshtein distance ([#10](https://github.com/Isvane/fuzzies/pull/10))
- *(search)* add dynamic limit to SearchBuilder ([#7](https://github.com/Isvane/fuzzies/pull/7))
- *(search)* [**breaking**] implement Builder pattern for dictionary search ([#3](https://github.com/Isvane/fuzzies/pull/3))

### Other

- *(dictionary)* make Dictionary fields private ([#13](https://github.com/Isvane/fuzzies/pull/13))
- add public API docstrings and update README examples ([#12](https://github.com/Isvane/fuzzies/pull/12))
- accept generic impl AsRef<Path> in Dictionary::open and build ([#9](https://github.com/Isvane/fuzzies/pull/9))
- *(build)* move build to a Dictionary ([#8](https://github.com/Isvane/fuzzies/pull/8))
- enhance README and update example ([#6](https://github.com/Isvane/fuzzies/pull/6))
- add release-plz workflow ([#4](https://github.com/Isvane/fuzzies/pull/4))
- include README in crate documentation ([#1](https://github.com/Isvane/fuzzies/pull/1))
