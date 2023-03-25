/// Parses a string array.
pub(crate) fn parse_string_array(s: &str) -> Vec<&str> {
    if s.contains(',') {
        const LEFT_BRACKETS: [char; 4] = ['(', '[', '{', '<'];
        const RIGHT_BRACKETS: [char; 4] = [')', ']', '}', '>'];
        let values = if s.contains(LEFT_BRACKETS) && s.contains(RIGHT_BRACKETS) {
            let bracket_indices = s
                .match_indices(&LEFT_BRACKETS)
                .map(|(index, _)| index)
                .zip(s.match_indices(&RIGHT_BRACKETS).map(|(index, _)| index))
                .collect::<Vec<_>>();
            let comma_indices = s
                .match_indices(',')
                .filter_map(|(index, _)| {
                    if bracket_indices
                        .iter()
                        .any(|(start, end)| index > *start && index < *end)
                    {
                        None
                    } else {
                        Some(index)
                    }
                })
                .collect::<Vec<_>>();
            let mut values = Vec::new();
            let mut current = 0;
            for index in comma_indices {
                if let Some(value) = s.get(current..index) {
                    values.push(value);
                }
                current = index + 1;
            }
            if let Some(value) = s.get(current..) {
                values.push(value);
            }
            values
        } else {
            s.split(',').collect::<Vec<_>>()
        };
        values
    } else {
        vec![s]
    }
}

#[cfg(test)]
mod tests {
    use super::parse_string_array;

    #[test]
    fn it_parses_string_array() {
        assert_eq!(
            parse_string_array("id,name,array_length(roles,1)=>roles_count"),
            vec!["id", "name", "array_length(roles,1)=>roles_count"],
        );
    }
}
