use std::error::Error;
use std::io::{stdin, stdout, Write};
use std::process::Command;

fn main() -> Result<(), Box<dyn Error>> {
    loop {
        print!("rum> ");
        stdout().flush()?;

        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();

        let command = input.trim();

        let mut child = Command::new(command).spawn().unwrap();

        child.wait()?;
    }
}
