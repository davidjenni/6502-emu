
; Euclid's GCD using subtraction only
; compiled via https://www.masswerk.at/6502/assembler.html

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
