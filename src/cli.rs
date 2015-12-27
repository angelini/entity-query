use std::io;

#[derive(Debug)]
pub enum CLICommand {
    Load(String),
    Query(String),
}

#[derive(Debug)]
pub enum ParseError {
    IOError,
    InvalidCommand(String),
}

fn read_input() -> Result<String, io::Error> {
    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(_) => Ok(input),
        Err(e) => Err(e),
    }
}

pub fn read<'a>() -> Result<CLICommand, ParseError> {
    let input = match read_input() {
        Ok(input) => input,
        Err(_) => return Err(ParseError::IOError),
    };
    let command = &input[..2];
    let args = input.chars().skip(2).filter(|c| *c != '\n').collect();

    if command == "l " {
        Ok(CLICommand::Load(args))
    } else if command == "q " {
        Ok(CLICommand::Query(args))
    } else {
        Err(ParseError::InvalidCommand(input.clone()))
    }
}
