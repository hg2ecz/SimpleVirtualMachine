include "stdlib.sc";

u8 source[16];
u8 copybuf[16];
u8 number_text[8];
u8 ring_data[8];
u16 ring_head;
u16 ring_tail;
u16 ring_out;

u16 main() {
    u16 crc;
    u16 parsed;

    store8(&source + 0, 49); // '1'
    store8(&source + 1, 50); // '2'
    store8(&source + 2, 51); // '3'
    store8(&source + 3, 52); // '4'
    store8(&source + 4, 0);

    memcpy(&copybuf, &source, 5);
    parsed = parse_u16_dec(&copybuf);
    u16_to_dec(&number_text, parsed + 1);

    crc = crc16_ccitt(&copybuf, 4);

    ring_init(&ring_head, &ring_tail);
    ring_push(&ring_data, 8, &ring_head, &ring_tail, 65);
    ring_push(&ring_data, 8, &ring_head, &ring_tail, 66);
    ring_pop(&ring_data, 8, &ring_head, &ring_tail, &ring_out);

    puts("value=");
    putstr(&number_text);
    puts(" crc=");
    puthex16(crc);
    newline();

    return load8(&ring_out);
}
