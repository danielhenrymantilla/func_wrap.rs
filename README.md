# `::func_wrap`

[![Repository](https://img.shields.io/badge/repository-GitHub-brightgreen.svg)](https://github.com/danielhenrymantilla/func_wrap.rs)
[![Latest version](https://img.shields.io/crates/v/func_wrap.svg)](https://crates.io/crates/func_wrap)
[![Documentation](https://docs.rs/func_wrap/badge.svg)](https://docs.rs/func_wrap)
[![MSRV](https://img.shields.io/badge/MSRV-1.42.0-white)](https://gist.github.com/danielhenrymantilla/8e5b721b3929084562f8f65668920c33)
[![License](https://img.shields.io/crates/l/func_wrap.svg)](https://github.com/danielhenrymantilla/func_wrap.rs/blob/master/Cargo.toml#L7)
[![CI](https://github.com/danielhenrymantilla/func_wrap.rs/workflows/CI/badge.svg)](https://github.com/danielhenrymantilla/func_wrap.rs/actions)

Helper crate for procedural macro authors that wish to duplicate some
received function inside its body, so as to be able to _wrap_ with some
prologue, epilogue, cache-ing, _etc._

## Examples

See https://docs.rs/require_unsafe_in_body for a real-life example of using
it.

[https://docs.rs/require_unsafe_in_body]: https://docs.rs/require_unsafe_in_body
