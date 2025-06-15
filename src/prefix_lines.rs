pub trait PrefixLines {
    fn prefix_lines(&self, prefix: &str) -> String;
}

impl PrefixLines for str {
    fn prefix_lines(&self, prefix: &str) -> String {
        self.lines()
            .map(|line| format!("{}{}", prefix, line))
            .collect::<Vec<_>>()
            .join("\n")
    }
}
