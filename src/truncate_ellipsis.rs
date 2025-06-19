pub trait TruncateEllipsis {
    fn truncate_ellipsis(&self, max_len: usize) -> String;
}


impl TruncateEllipsis for str {
    /// Truncate a string to a maximum length, adding an ellipsis if the string is longer than
    /// `max_len`. The string is truncated to the number UTF-8 characters so it may be longer in
    /// bytes than what the `max_len` specifies.
    ///
    /// # Example
    ///
    /// ```
    /// # use ahiru_tpm::truncate_ellipsis::*;
    /// # fn main() {
    /// let str = "string with 20 chars";
    /// let str_truncated = str.truncate_ellipsis(12);
    /// assert_eq!(str_truncated, "string with…");
    /// # }
    /// ```
    fn truncate_ellipsis(&self, max_len: usize) -> String {
        assert!(max_len > 0, "The max length must be greater than 0");

        //let char_indices = self.char_indices().try_len().expect("A str should have a known length");
        let mut char_indices = self.char_indices();

        // The last char when an ellipsis is applied. It's one less than the last possible char
        // when the string is exactly max_len and no ellipsis needs to be added.
        let last_char_with_ellipsis = char_indices.nth(max_len - 2);
        
        // The first truncated char. If this is `Some`, the string is longer than max_len.
        let truncated_char = char_indices.nth(1);

        match truncated_char {
            // When the string is longer, we need to add an ellipsis:
            Some(_) => {
                let (n, _) = last_char_with_ellipsis.expect("If truncated char is `Some`, the chars before it should also be `Some`");
                let n = n+1;
                let truncated = &self[..n];
                format!("{}…", truncated)
            }

            // When the string is exactly max_len or shorter, we can just return a copy:
            None => self.to_string(),
        }

    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_not_truncate_shorter_strings() {
        let str = "string with 20 chars";
        let str_truncated = str.truncate_ellipsis(21);

        assert_eq!(str_truncated, str);
    }

    #[test]
    fn should_not_truncate_string_with_exactly_max_length() {
        let str = "string with 20 chars";
        let str_truncated = str.truncate_ellipsis(20);

        assert_eq!(str_truncated, str);
    }

    #[test]
    fn should_truncate_longer_strings() {
        let str = "string with 20 chars";
        let str_truncated = str.truncate_ellipsis(19);

        assert_eq!(str_truncated, "string with 20 cha…");
    }

    #[test]
    #[should_panic(expected = "The max length must be greater than 0")]
    fn zero_max_length() {
        "string with 20 chars".truncate_ellipsis(0);
    }

    #[test]
    fn should_correctly_truncate_strings_with_utf8_chars() {
        let str = "string with 82 chars and fancy 'ƒ' with two-bytes length, making it 83 bytes long.";
        let str_truncated = str.truncate_ellipsis(81);

        // Original string should be 83 bytes but only 82 chars long
        assert_eq!(str.len(), 83);
        assert_eq!(str.chars().count(), 82);

        assert_eq!(str_truncated, "string with 82 chars and fancy 'ƒ' with two-bytes length, making it 83 bytes lon…");

        // Truncated string contains a fancy "f" (two bytes) and an ellipsis char (three bytes) so
        // it should have three more bytes than it has chars.
        assert_eq!(str_truncated.len(), 84);
        assert_eq!(str_truncated.chars().count(), 81);
    }
}
