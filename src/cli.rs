use linenoise;

#[derive(Debug)]
pub enum CLICommand {
    Load(String),
    LoadCSV(String, String, String),
    Query(String),
    Write(String),
    Empty,
    None,
    Exit,
}

#[derive(Debug)]
pub enum ParseError {
    InvalidCommand(String),
    InvalidArgs(String),
}

pub fn read<'a>() -> Result<CLICommand, ParseError> {
    let input = match linenoise::input("> ") {
        Some(i) => i,
        None => return Ok(CLICommand::None),
    };

    // Save before adding to the history to avoid saving the last "exit"
    linenoise::history_save(".history");
    linenoise::history_add(&input);

    if input == "" {
        return Ok(CLICommand::None);
    }

    let split = input.split(" ").collect::<Vec<&str>>();
    let command = split[0];
    let args = split.into_iter().skip(1).collect::<Vec<&str>>();
    let all_args = args.join(" ");

    match command {
        "l" => Ok(CLICommand::Load(all_args)),
        "c" => {
            if args.len() == 3 {
                Ok(CLICommand::LoadCSV((*args.get(0).unwrap()).to_string(),
                                       (*args.get(1).unwrap()).to_string(),
                                       (*args.get(2).unwrap()).to_string()))
            } else {
                Err(ParseError::InvalidArgs(all_args))
            }
        }
        "q" => Ok(CLICommand::Query(all_args)),
        "w" => Ok(CLICommand::Write(all_args)),
        "empty" => Ok(CLICommand::Empty),
        "clear" => {
            linenoise::clear_screen();
            Ok(CLICommand::None)
        }
        "exit" => Ok(CLICommand::Exit),
        _ => Err(ParseError::InvalidCommand(command.to_string())),
    }
}
