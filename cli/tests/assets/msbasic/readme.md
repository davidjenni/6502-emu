# Microsoft BASIC for 6502 CPUs

Bin files and assembly source files are found at:

- <https://github.com/mist64/msbasic>
- Bin files: <https://github.com/mist64/msbasic/tree/master/orig>
- [Credits](https://github.com/mist64/msbasic#credits)

## KERNAL basic

basic4.prg is for VIC, start address: 0xd3b5 (dec 54197)
Since the init stage also does a destructive memory test to determine RAM size, set the `--read_only` flag to simulate ROM.

```bash
> cargo run --bin r6502 -- debug -b ./cli/tests/assets/msbasic/basic4.prg -s 0xD3B5 -r
```

<https://github.com/mist64/kernalemu>

## Running in r6502 (experimental!)

The CBM1/2 bin images expect to be loaded into 0xC000 (dec 49152), e.g.:

```bash
> cargo run --bin r6502 -- debug -b ./cli/tests/assets/msbasic/cbmbasic1.bin -l 0xC000 -r
```
