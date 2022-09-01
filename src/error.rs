/// The Error returned by scanf if the regex doesn't match the input.
#[derive(Debug, Clone, Copy)]
pub struct RegexMatchFailed;

impl std::error::Error for RegexMatchFailed {}

impl std::fmt::Display for RegexMatchFailed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "scanf: The regex did not match the input")
    }
}
