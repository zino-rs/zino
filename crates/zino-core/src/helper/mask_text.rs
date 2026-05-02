/// Masks text with masking options.
pub(crate) fn mask_text(text: &str, num_prefix_chars: usize, num_suffix_chars: usize) -> String {
    let char_count = text.chars().count();
    let suffix_index = char_count.saturating_sub(num_suffix_chars);
    let mut masked_text = String::with_capacity(text.len());
    for (i, c) in text.chars().enumerate() {
        if i < num_prefix_chars || i >= suffix_index {
            masked_text.push(c);
        } else {
            masked_text.push('*');
        }
    }
    masked_text
}
