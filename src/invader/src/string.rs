/// Get the slice up to the terminator, returning the full slice if no terminator exists.
pub fn to_terminator<T: std::cmp::PartialEq>(input: &[T], terminator: T) -> &[T] {
    for i in 0..input.len() {
        if input[i] == terminator {
            return &input[..i];
        }
    }
    input
}
