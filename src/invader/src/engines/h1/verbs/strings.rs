pub fn strings_verb(verb: &crate::cmd::Verb, args: &[&str], executable: &str) -> crate::engines::ExitCode {
    super::unicode_strings::unicode_strings_verb(verb, args, executable)
}
