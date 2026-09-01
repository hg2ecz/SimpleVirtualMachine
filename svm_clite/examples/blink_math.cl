fn mul_add(u16 a, u16 b, u16 c) -> u16 {
    return a * b + c;
}

fn main() -> u16 {
    u16 x = 5;
    u16 y = 7;
    return mul_add(x, y, 3);
}
