# cargo-vs • autogenerate visual studio solutions / projects

[![GitHub](https://img.shields.io/github/stars/MaulingMonkey/cargo-vs.svg?label=GitHub&style=social)](https://github.com/MaulingMonkey/cargo-vs)
[![crates.io](https://img.shields.io/crates/v/cargo-vs.svg)](https://crates.io/crates/cargo-vs)
[![%23![forbid(unsafe_code)]](https://img.shields.io/github/search/MaulingMonkey/cargo-vs/unsafe%2bextension%3Ars?color=green&label=%23![forbid(unsafe_code)])](https://github.com/MaulingMonkey/cargo-vs/search?q=forbid%28unsafe_code%29+extension%3Ars)
[![rust: stable](https://img.shields.io/badge/rust-stable-yellow.svg)](https://gist.github.com/MaulingMonkey/c81a9f18811079f19326dac4daa5a359#minimum-supported-rust-versions-msrv)
[![License](https://img.shields.io/crates/l/cargo_vs.svg)](https://github.com/MaulingMonkey/cargo-vs)
[![Build Status](https://github.com/MaulingMonkey/cargo-vs/workflows/Rust/badge.svg)](https://github.com/MaulingMonkey/cargo-vs/actions?query=workflow%3Arust)



<h2 name="quickstart">Quickstart</h2>

```cmd
cd my-rust-project
cargo install cargo-vs
cargo vs2017
"%ProgramFiles(x86)%\Microsoft Visual Studio\2017\Community\Common7\IDE\devenv.exe" vs\vs2017.sln
```



<h2 name="generated">What's generated?</h2>

`vs/.gitignore` since many/most projects don't want .vsode boilerplate checked in IME (although I always provide mine)<br>
`vs/vs2017.sln`<br>
`vs/vs2017/*.vcsproj` - Makefile style projects which will invoke `cargo +stable-%ARCH%-pc-windows-msvc build --target %ARCH%-pc-windows-msvc --package [package] [--bin|--example] [target] [--release]`<br>



<h2 name="caveat-32-bit-toolchains">Caveat: 32-bit (x86/Win32) builds require an i686 toolchain</h2>

When MSVC configures a build environment, rustc will pick up the `%PATH%` provided `link.exe`.<br>
On the plus side, this means it should work for new VS versions rustc doesn't recognize.<br>
On the minus side, this means that `build.rs` and your target use the same linker, and must match architectures.<br>
This could be "fixed" by clearing a bunch of environment variables, but...



<h2 name="why-not-dot-vs">Why makefile projects instead of .vs/*.vs.json?</h2>

* Easier to retrofit support for legacy Visual Studio versions
* Presumably easier to integrate into your existing C++/C# msuild mess
* I can't figure out how to launch the graphics debugger with .vs/*
* Proper build matricies



<h2 name="license">License</h2>

Licensed under either of

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.



<h2 name="contribution">Contribution</h2>

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
