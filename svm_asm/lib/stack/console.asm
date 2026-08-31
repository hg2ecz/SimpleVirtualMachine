\ Stack ISA console helpers. Console byte MMIO = 0xFF20.
\ ABI: putc ( ch -- ); puts ( ram-addr -- ); newline ( -- ).
.proc putc
    0xFF20 STORE8
    RET
.endproc

.proc newline
    13 0xFF20 STORE8
    10 0xFF20 STORE8
    RET
.endproc

.proc puts
puts_loop:
    DUP LOAD8
    DUP JZ puts_done
    0xFF20 STORE8
    1 ADD
    JMP puts_loop
puts_done:
    DROP
    DROP
    RET
.endproc
