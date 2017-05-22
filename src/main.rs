use std::io::Write;
use std::io::Read;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Token {
    Increment,
    Decrement,
    ShiftLeft,
    ShiftRight,
    Input,
    Output,
    BeginLoop,
    EndLoop,
}

#[derive(Clone, Debug)]
pub enum SyntaxItem {
    Single(Token),
    Loop(Vec<SyntaxItem>),
}

#[derive(Clone, Debug)]
pub struct State {
    data: Vec<u8>,
    pointer: usize,

    commands: Vec<SyntaxItem>,
}

impl State {
    pub fn new(commands: Vec<SyntaxItem>) -> Self {
        let mut state = State {
            data: Vec::new(),
            pointer: 0,

            commands: commands,
        };
        state.data.push(0);
        state
    }
}

fn lex(input: &String) -> Vec<Token> {
    input.chars()
        .filter_map(|c| match c {
            '+' => Some(Token::Increment),
            '-' => Some(Token::Decrement),
            '<' => Some(Token::ShiftLeft),
            '>' => Some(Token::ShiftRight),
            ',' => Some(Token::Input),
            '.' => Some(Token::Output),
            '[' => Some(Token::BeginLoop),
            ']' => Some(Token::EndLoop),
            _ => None,
        })
        .collect()
}

fn parse(input: &String) -> Result<Vec<SyntaxItem>, String> {
    let tokens = lex(input);

    let mut tree = Vec::new();
    let mut it = tokens.iter().enumerate();
    while let Some((i, token)) = it.next() {
        match *token {
            Token::BeginLoop => {
                // Cut off already processed tokens.
                let mut inner = &input[i + 1..];

                let mut counter = 1;
                let mut index = 0;
                for c in inner.chars() {
                    if c == '[' {
                        counter = counter + 1;
                    } else if c == ']' {
                        counter = counter - 1;
                    }

                    index += 1;
                    if counter == 0 {
                        break;
                    }
                }

                if counter != 0 {
                    panic!("Unmatched parenthesis found.");
                }

                inner = &inner[..index];

                // Parse inner tokens.
                let item = parse(&inner.to_owned())
                    .expect("Could not process inner structure of loop.");

                tree.push(SyntaxItem::Loop(item));

                // This looks really weird.
                for _ in 0..index {
                    it.next();
                }
            }
            Token::EndLoop => continue,
            _ => tree.push(SyntaxItem::Single(*token)),
        }
    }

    Ok(tree)
}

fn run(state: &mut State) {
    let mut it = state.commands.iter();
    while let Some(command) = it.next() {
        // println!("Executing: {:?}", *command);
        match *command {
            SyntaxItem::Single(ref t) => {
                match *t {
                    Token::Increment => {
                        if state.data[state.pointer] == 255 {
                            state.data[state.pointer] = 0;
                        } else {
                            state.data[state.pointer] += 1
                        }
                    }
                    Token::Decrement => {
                        if state.data[state.pointer] == 0 {
                            state.data[state.pointer] = 255;
                        } else {
                            state.data[state.pointer] -= 1
                        }
                    }
                    Token::ShiftLeft => {
                        if state.pointer > 0 {
                            state.pointer = state.pointer - 1;
                        } else {
                            state.data.insert(0, 0);
                        }
                    }
                    Token::ShiftRight => {
                        state.data.push(0);
                        state.pointer = state.pointer + 1;
                    }
                    Token::Input => {
                        let mut s = String::new();
                        std::io::stdin().read_line(&mut s).expect("Unable to read from STDIN.");

                        let trim = s.trim();
                        let result = trim.parse::<u8>();
                        match result {
                            Ok(i) => {
                                state.data[state.pointer] = i;
                                continue;
                            }
                            Err(_) => (),
                        }

                        let option = trim.chars().nth(0);
                        match option {
                            Some(c) => {
                                state.data[state.pointer] = c as u8;
                                continue;
                            }
                            None => panic!("Could not parse input."),
                        }
                    } 
                    Token::Output => {
                        print!("{}", state.data[state.pointer] as char);
                        std::io::stdout().flush().expect("Could not flush.");
                    }
                    Token::BeginLoop => continue,
                    Token::EndLoop => continue,
                }
            }
            SyntaxItem::Loop(ref v) => {
                let mut s = state.clone();
                s.commands = v.clone();

                while state.data[state.pointer] != 0 {
                    run(&mut s);

                    state.data = s.data.clone();
                    state.pointer = s.pointer;
                }
            }
        }
    }
}


fn main() {
    let args: Vec<String> = std::env::args().collect();

    let filename = &args[1];

    let mut f = std::fs::File::open(filename).expect("File not found.");

    let mut contents = String::new();
    f.read_to_string(&mut contents).expect("Could not read file.");

    contents = contents.chars().filter(|c| !c.is_whitespace()).collect();

    let result = parse(&contents).expect("Could not parse.");

    let mut state = State::new(result);
    run(&mut state);
}
