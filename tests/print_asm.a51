.org 00H

start:
    lcall print
    ljmp shutdown

print:
    mov r0, #0x99
    mov @r0, #'H'
    mov @r0, #'e'
    mov @r0, #'l'
    mov @r0, #'l'
    mov @r0, #'o'
    mov @r0, #'\n'
    ret

shutdown:
    mov dptr, #0xFFFF
    mov a, #1
    movx @dptr, a
