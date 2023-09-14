# Calculate GDC via Euclid's substraction algorithm

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

```code
; zero page addresses:
VAR_A = $40
VAR_B = $41

.org $0600
; start:
0600 A5 40      LDA VAR_A
; diff:
0602 38         SEC
0603 E5 41      SBC VAR_B
0605 F0 12      BEQ done
0607 30 05      BMI swap
0609 85 40      STA VAR_A
060B 4C 02 06   JMP diff
; swap
060E A6 40      LDX VAR_A
0610 A4 41      LDY VAR_B
0612 86 41      STX VAR_B
0614 84 40      STY VAR_A
0616 4C 00 06   JMP start
; done:
0619 A5 40      LDA VAR_A
061B 00         BRK
```
