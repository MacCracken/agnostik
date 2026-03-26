# Contributing to Agnostik

1. Fork, branch, `make check`, add tests, submit PR.
2. All public types must be Serialize + Deserialize with roundtrip tests.
3. All public enums must be `#[non_exhaustive]`.
4. No `unwrap()` or `panic!()` in library code.
