// bool is logical in the language and one byte in memory.
fn less(u16 a, u16 b) -> bool {
    return a < b;
}

fn main() -> u16 {
    bool ready = less(10, 20);
    if (ready) {
        return 1;
    }
    return 0;
}
