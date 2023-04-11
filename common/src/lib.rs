/// Example of a common function
/// # Arguments
/// * `left` - The left side of the addition
/// * `right` - The right side of the addition
/// # Returns
/// The sum of the two numbers
pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
