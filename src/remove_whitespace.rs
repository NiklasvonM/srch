/// Removes whitespace characters from a string.
///
/// This function iterates through the input string and creates a new string
/// containing only the non-whitespace characters. Whitespace characters are
/// defined by Unicode's definition of whitespace.
///
/// # Examples
///
/// ```
/// let text = "  Hello  World!  ";
/// let result = remove_whitespace(text);
/// assert_eq!(result, "HelloWorld!");
///
/// let text_with_tabs_and_newlines = "Hello\tWorld\n!";
/// let result = remove_whitespace(text_with_tabs_and_newlines);
/// assert_eq!(result, "HelloWorld!");
/// ```
pub fn remove_whitespace(text: &str) -> String {
    let mut result = String::new();
    for char in text.chars() {
        if !char.is_whitespace() {
            result.push(char);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_whitespace() {
        let text = "  Hello  World!  ";
        let result = remove_whitespace(text);
        assert_eq!(result, "HelloWorld!");

        let text_with_tabs_and_newlines = "Hello\tWorld\n!";
        let result = remove_whitespace(text_with_tabs_and_newlines);
        assert_eq!(result, "HelloWorld!");

        let text_no_whitespace = "HelloWorld!";
        let result = remove_whitespace(text_no_whitespace);
        assert_eq!(result, "HelloWorld!");

        let empty_text = "";
        let result = remove_whitespace(empty_text);
        assert_eq!(result, "");
    }
}