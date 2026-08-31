// Byte ring-buffer helpers. One slot remains unused to distinguish full/empty.
// head_addr and tail_addr point to u16 indices. capacity must be >= 2.

void ring_init(u16 head_addr, u16 tail_addr) {
    store16(head_addr, 0);
    store16(tail_addr, 0);
}

u16 ring_empty(u16 head_addr, u16 tail_addr) {
    return load16(head_addr) == load16(tail_addr);
}

u16 ring_count(u16 capacity, u16 head_addr, u16 tail_addr) {
    u16 head;
    u16 tail;
    head = load16(head_addr);
    tail = load16(tail_addr);
    if (head >= tail) return head - tail;
    return capacity - tail + head;
}

u16 ring_full(u16 capacity, u16 head_addr, u16 tail_addr) {
    u16 head;
    u16 next;
    head = load16(head_addr);
    next = head + 1;
    if (next >= capacity) next = 0;
    return next == load16(tail_addr);
}

// Returns 1 on success, 0 if full.
u16 ring_push(u16 data, u16 capacity, u16 head_addr, u16 tail_addr, u16 value) {
    u16 head;
    u16 next;
    if (capacity < 2) return 0;
    head = load16(head_addr);
    next = head + 1;
    if (next >= capacity) next = 0;
    if (next == load16(tail_addr)) return 0;
    store8(data + head, value);
    store16(head_addr, next);
    return 1;
}

// Returns 1 on success and writes one byte to out_addr; 0 if empty.
u16 ring_pop(u16 data, u16 capacity, u16 head_addr, u16 tail_addr, u16 out_addr) {
    u16 tail;
    u16 next;
    if (capacity < 2) return 0;
    tail = load16(tail_addr);
    if (tail == load16(head_addr)) return 0;
    store8(out_addr, load8(data + tail));
    next = tail + 1;
    if (next >= capacity) next = 0;
    store16(tail_addr, next);
    return 1;
}
