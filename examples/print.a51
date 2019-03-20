.org 00H

start:
    lcall print
    ljmp shutdown

print:
    mov dptr, #0x400
    mov a, #'H'
    movx @dptr, a
    mov a, #'e'
    movx @dptr, a
    mov a, #'l'
    movx @dptr, a
    mov a, #'l'
    movx @dptr, a
    mov a, #'o'
    movx @dptr, a
    ret

shutdown:
    mov dptr, #0xFFFF
    mov a, #1
    movx @dptr, a
