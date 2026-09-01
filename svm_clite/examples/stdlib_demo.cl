include "math.cl";
include "memory.cl";

u8 data[4];

fn main() -> u16 {
    mem_zero(&data[0], 4);
    data[0] = 84;
    data[1] = 30;
    return gcd_u16(data[0], data[1]);
}
