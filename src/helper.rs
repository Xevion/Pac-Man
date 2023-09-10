pub fn is_adjacent(a: (u32, u32), b: (u32, u32), diagonal: bool) -> bool {
    let (ax, ay) = a;
    let (bx, by) = b;
    if diagonal {
        (ax == bx && (ay == by + 1 || ay == by - 1))
            || (ay == by && (ax == bx + 1 || ax == bx - 1))
            || (ax == bx + 1 && ay == by + 1)
            || (ax == bx + 1 && ay == by - 1)
            || (ax == bx - 1 && ay == by + 1)
            || (ax == bx - 1 && ay == by - 1)
    } else {
        (ax == bx && (ay == by + 1 || ay == by - 1))
            || (ay == by && (ax == bx + 1 || ax == bx - 1))
    }
}