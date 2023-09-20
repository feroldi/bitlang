use crate::driver;

mod test_basic_programs;
mod test_binding;
mod test_function_call;
mod test_if_else;

fn compile(source_code: &str) -> String {
    driver::compile(&strip_margin(source_code))
}

fn check<S: AsRef<str>>(program: S, expected_program: &str) {
    use pretty_assertions::assert_eq;

    assert_eq!(
        program.as_ref().trim(),
        strip_margin(expected_program).trim()
    );
}

fn strip_margin(text: &str) -> String {
    text.split('\n')
        .map(|line| {
            let mut stripped_line = line.chars().skip_while(|&c| c != '|');
            stripped_line.next();

            stripped_line.collect::<String>()
        })
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}
