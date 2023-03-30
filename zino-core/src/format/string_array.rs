/// Parses a string array.
pub(crate) fn parse_string_array(s: &str) -> Vec<&str> {
    let mut string_array = Vec::new();
    let mut chars_stack = Vec::new();
    let mut last_index = 0;
    for (i, c) in s.chars().enumerate() {
        match c {
            '(' | '[' | '{' => {
                chars_stack.push(c);
            }
            ')' => {
                if chars_stack.pop() != Some('(') {
                    return string_array;
                }
            }
            ']' => {
                if chars_stack.pop() != Some('[') {
                    return string_array;
                }
            }
            '}' => {
                if chars_stack.pop() != Some('{') {
                    return string_array;
                }
            }
            ',' => {
                if chars_stack.is_empty() {
                    string_array.push(&s[last_index..i]);
                    last_index = i + 1;
                }
            }
            _ => {}
        }
    }
    if !chars_stack.is_empty() {
        return string_array;
    }
    string_array.push(&s[last_index..]);
    string_array
}

#[cfg(test)]
mod tests {
    use super::parse_string_array;

    #[test]
    fn it_parses_string_array() {
        assert_eq!(
            parse_string_array("id,name,array_length(roles,1)->roles_count"),
            vec!["id", "name", "array_length(roles,1)->roles_count"],
        );
    }
}
