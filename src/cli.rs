use linenoise;

#[derive(Debug)]
pub struct Join(String, String);

#[derive(Debug)]
pub enum CLICommand {
    Load(String),
    LoadCSV(String, String, String, Vec<Join>),
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
    InvalidJoinClause(String),
}

pub fn read() -> Result<CLICommand, ParseError> {
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
            if args.len() >= 3 {
                Ok(CLICommand::LoadCSV((*args.get(0).unwrap()).to_owned(),
                                       (*args.get(1).unwrap()).to_owned(),
                                       (*args.get(2).unwrap()).to_owned(),
                                       try!(parse_joins(&args[3..].join(" ")))))
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
        _ => Err(ParseError::InvalidCommand(command.to_owned())),
    }
}

// c data/artists.csv artist Year
// c data/albums.csv album Year join(Artist, "a=artist/name")
// c data/tracks.csv track Year join(Artist, "a=artist/name") join(Album, "a=album/name")

fn parse_joins(raw: &str) -> Result<Vec<Join>, ParseError> {
    let join_re = regex!(r#"(\S+),\s+"(.*)"\)"#);

    raw.split("join(")
       .map(|s| s.trim())
       .filter(|s| *s != "")
       .map(|clause| {
           println!("clause {}", clause);

           if let Some(caps) = join_re.captures(clause) {
               Ok(Join(caps.at(1).unwrap().to_owned(),
                       caps.at(2).unwrap().to_owned()))
           } else {
               Err(ParseError::InvalidJoinClause(clause.to_owned()))
           }
       })
       .collect()
}
