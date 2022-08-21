use std::process::ExitCode;

extern crate invader;

fn main() -> ExitCode {
    invader::cmd::main_fn(&invader::h1::HaloCE::default())
}
