# 6502 CPU emulator in rust

[![CI](https://github.com/davidjenni/6502-emu/actions/workflows/CI.yml/badge.svg)](https://github.com/davidjenni/6502-emu/actions/workflows/CI.yml)
[![codecov](https://codecov.io/gh/davidjenni/6502-emu/graph/badge.svg?token=K65NNITM7V)](https://codecov.io/gh/davidjenni/6502-emu)

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

## Run via cargo

```bash
A 6502 emulator written in Rust

Usage: r6502.exe [OPTIONS] [COMMAND]

Arguments:
  [COMMAND]
          [default: run]
          [possible values: run, debug]

Options:
  -b, --binary <BINARY>
          Path to binary file to load and run

  -f <FORMAT>
          File format of the binary file to load

          Possible values:
          - bin: Plain binary with no header, little endian byte order
          - prg: Like a bin file, but with a 16 byte header that indicates the load address

  -l, --load-address <LOAD_ADDRESS>
          Load address (u16) for binary to be loaded to (inferred for .prg); if no start_addr it is also used as start address

  -s, --start-address <START_ADDRESS>
          Start address (u16) for binary to be started with; can be hex address in 0x1234 format

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version

```

With an empty program, the reset vector 0xFFFE points to a BRK instruction,
halting the "program" after one instruction.

```bash
cargo run --bin r6502 --
No binary file specified, running empty program with single BRK instruction

PC: FFFE: A: 00 X: 00 Y: 00 S: 00000000 SP: 01FC
Instructions: 1; Cycles: 7; Clock speed: 1.045 MHz
Program finished after 6 μs:
done.
```

Debugging with step and disassembly listing is also possible.

```bash
cargo run --bin r6502 -- debug -b ./cli/tests/assets/euclid_gcd.prg -s 0x0200
Loaded 476 bytes at address 0040
PC: 0200: A: 00 X: 00 Y: 00 S: 00000000 SP: 01FF
(dbg)> di
  0200 LDA $40
  0202 SEC
  0203 SBC $41
  0205 BEQ $12 (0219)
  0207 BMI $05 (020E)
  0209 STA $40
  020B JMP $0202
  020E LDX $40
  0210 LDY $41
  0212 STX $41
(dbg)>
(dbg)> s
PC: 0202: A: 78 X: 00 Y: 00 S: 00000000 SP: 01FF
(dbg)>
PC: 0203: A: 78 X: 00 Y: 00 S: 00000001 SP: 01FF
(dbg)> h
Usage:
  step (s)          - step one instruction
  disassemble (di)  - disassemble instructions from current PC
  continue (c)      - continue execution
  <empty line>      - repeat last command
  quit (q)          - quit debugger
(dbg)>
```

## Feedback & Questions

Please use the issues tracker in the home repo: <https://github.com/davidjenni/6502-emu/issues>

## Contributing

This project will welcome contributions in the near future. At this stage, we're not ready for contributions,
but do welcome your suggestions via this repository's issue tracker.

Information on how to setup a local dev environment is also detailed in this document.

See details in [CONTRIBUTING](CONTRIBUTING.md)

### Code of Conduct

See details in [CODE_OF_CONDUCT](CODE_OF_CONDUCT.md)
