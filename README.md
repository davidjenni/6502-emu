# 6502 CPU emulator in rust

[![CI](https://github.com/davidjenni/6502-emu/actions/workflows/CI.yml/badge.svg)](https://github.com/davidjenni/6502-emu/actions/workflows/CI.yml)

![MOS6502 in a 40 pin plastic package](/assets/MOS%206502%20DIP40-small.jpg)
![MOS6502 die](/assets/MOS_6502_die-small.jpg)

The [MOS 6502 CPU](https://en.wikipedia.org/wiki/MOS_Technology_6502) spawned
the microcomputer age in the mid 1970s, and was the common runtime for
one of MSFT's earliest products, MSBASIC.
While the original first MS product targeted the Altair 8080,
it was the abundance of 6502-based computers (Apple I & II, PET, CBM, etc.)
that made MSBASIC truly popular and considerably helped the MSFT brand in its early days.

Running MSBASIC on a rust-based CPU emulator on a VM in the Azure cloud is a geeky,
but also educational showcase how the whole computer industry has evolved over the past 50 years.

![rust crustacean](/assets/crabby.png)

This project is also my first foray into programming with rust; the code in this repo is the journey
to get hopefully close to idiomatic rust programming.
My notes on and sources for [the journey to rust](/docs/Learning_Rust.md).

## Feedback & Questions

Please use the issues tracker in the home repo: <https://github.com/davidjenni/6502-emu/issues>

## Contributing

This project will welcome contributions in the near future. At this stage, we're not ready for contributions,
but do welcome your suggestions via this repository's issue tracker.

Information on how to setup a local dev environment is also detailed in this document.

See details in [CONTRIBUTING](CONTRIBUTING.md)

### Code of Conduct

See details in [CODE_OF_CONDUCT](CODE_OF_CONDUCT.md)
