# 6502 CPU emulator in rust

[![CI](https://github.com/davidjenni/6502-emu/actions/workflows/CI.yml/badge.svg)](https://github.com/davidjenni/6502-emu/actions/workflows/CI.yml)

The [MOS 6502 CPU](https://en.wikipedia.org/wiki/MOS_Technology_6502) spawned
the microcomputer age in the mid 1970s, and was the common runtime for
one of MSFT's earliest products, MSBASIC.
While the original first MS product targeted the Altair 8080,
it was the abundance of 6502-based computers (Apple I & II, PET, CBM, etc.)
that made MSBASIC truly popular and considerably helped the MSFT brand in its early days.

Running MSBASIC on a rust-based CPU emulator on a VM in our Azure cloud is a geeky, but also educational showcase how the whole computer industry has evolved over the past 50 years.

![MOS6502 in a 40 pin plastic package](/assets/MOS%206502%20DIP40.jpg)

## Build and test

```bash
./ci.sh
```
