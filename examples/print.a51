.org 00H

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

mov dptr, #0xFFFF
mov a, #1
movx @dptr, a
