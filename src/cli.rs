use linenoise;

#[derive(Debug)]
pub enum CLICommand {
    Load(String),
    LoadCSV(String, String, String),
    Query(String),
    Write(String),
    Empty,
}

#[derive(Debug)]
pub enum ParseError {
    InvalidCommand(String),
    InvalidArgs(String),
}

pub fn read<'a>() -> Result<CLICommand, ParseError> {
    let input = match linenoise::input("> ") {
        Some(i) => i,
        None => return Ok(CLICommand::Empty),
    };

    linenoise::history_add(&input);

    let command = &input[..2];
    let args = input.chars().skip(2).filter(|c| *c != '\n').collect::<String>();
    let args_split = args.split(" ").collect::<Vec<&str>>();

    match command {
        "l " => Ok(CLICommand::Load(args_split.join(" "))),
        "c " => {
            if args_split.len() == 3 {
                Ok(CLICommand::LoadCSV((*args_split.get(0).unwrap()).to_string(),
                                       (*args_split.get(1).unwrap()).to_string(),
                                       (*args_split.get(2).unwrap()).to_string()))
            } else {
                Err(ParseError::InvalidArgs(args_split.join(" ")))
            }
        }
        "q " => Ok(CLICommand::Query(args_split.join(" "))),
        "w " => Ok(CLICommand::Write(args_split.join(" "))),
        _ => Err(ParseError::InvalidCommand(command.to_string())),
    }
}
