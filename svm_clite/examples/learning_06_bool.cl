fn less(u16 a, u16 b) -> bool {
    return a < b;
}

fn main() -> u16 {
    bool ready = less(1, 2);
    if (ready) {
        return 1;
    }
    return 0;
}
