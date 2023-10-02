# Microsoft BASIC for 6502 CPUs

Bin files and assembly source files are found at:

- <https://github.com/mist64/msbasic>
- Bin files: <https://github.com/mist64/msbasic/tree/master/orig>
- [Credits](https://github.com/mist64/msbasic#credits)


## Running in r6502 (experimental!)

The CBM1/2 bin images expect to be loaded into 0xC000 (dec 49152), e.g.:

```bash
> cargo run --bin r6502 -- debug -b ./cli/tests/assets/msbasic/cbmbasic1.bin -l 49152
```
