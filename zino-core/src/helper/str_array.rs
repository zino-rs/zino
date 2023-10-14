/// Parses a str array.
pub(crate) fn parse_str_array(s: &str) -> Vec<&str> {
    let mut str_array = Vec::new();
    let mut chars_stack = Vec::new();
    let mut last_index = 0;
    for (i, c) in s.chars().enumerate() {
        match c {
            '(' | '[' | '{' => {
                chars_stack.push(c);
            }
            ')' => {
                if chars_stack.pop() != Some('(') {
                    return str_array;
                }
            }
            ']' => {
                if chars_stack.pop() != Some('[') {
                    return str_array;
                }
            }
            '}' => {
                if chars_stack.pop() != Some('{') {
                    return str_array;
                }
            }
            ',' => {
                if chars_stack.is_empty() {
                    str_array.push(&s[last_index..i]);
                    last_index = i + 1;
                }
            }
            _ => {}
        }
    }
    if !chars_stack.is_empty() {
        return str_array;
    }
    str_array.push(&s[last_index..]);
    str_array
}

#[cfg(test)]
mod tests {
    use super::parse_str_array;

    #[test]
    fn it_parses_str_array() {
        assert_eq!(
            parse_str_array("id,name,roles_count:array_length(roles,1)"),
            vec!["id", "name", "roles_count:array_length(roles,1)"],
        );
    }
}
