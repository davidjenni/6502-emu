# Calculate GDC via Euclid's subtraction algorithm

<https://en.wikipedia.org/wiki/Euclidean_algorithm#Implementations>

## Basic algorithm

```code
function gcd(a, b)
  while a != b
    if a > b
       a := a - b
    else
       b := b - a
  return a
```

## Coded in 6502 assembly

Can use [online 6502 assembler](https://www.masswerk.at/6502/assembler.html) for small programs like this.

```code
; Euclid's GCD using subtraction only

.org $0040
VAR_A:
.byte $78 ; 126
VAR_B:
.byte 31  ; 49

.org $0200
start:
  LDA VAR_A
diff:
  SEC
  SBC VAR_B
  BEQ done
  BMI swap
  STA VAR_A
  JMP diff
swap:
  LDX VAR_A
  LDY VAR_B
  STX VAR_B
  STY VAR_A
  JMP start
done:
  LDA VAR_A
  BRK
```

Assembled:

```code
LOC   CODE         LABEL         INSTRUCTION

                   ; Euclid's GCD using subtraction only

0040                             * = $0040
0040               VAR_A
0040  78                         .BYTE $78 ; 126
0041               VAR_B
0041  1F                         .BYTE $1F ; 49
0200                             * = $0200
0200               START
0200  A5 40                      LDA $40
0202               DIFF
0202  38                         SEC
0203  E5 41                      SBC $41
0205  F0 12                      BEQ $0219
0207  30 05                      BMI $020E
0209  85 40                      STA $40
020B  4C 02 02                   JMP $0202
020E               SWAP
020E  A6 40                      LDX $40
0210  A4 41                      LDY $41
0212  86 41                      STX $41
0214  84 40                      STY $40
0216  4C 00 02                   JMP $0200
0219               DONE
0219  A5 40                      LDA $40
021B  00                         BRK
```

Hex code:

```code
0200: A5 40 38 E5 41 F0 12 30
0208: 05 85 40 4C 02 02 A6 40
0210: A4 41 86 41 84 40 4C 00
0218: 02 A5 40 00
```
