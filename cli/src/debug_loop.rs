use std::io;
use std::io::Write;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum DebuggerCommand {
    Step,
    List,
    // Memory,
    Continue,
    Quit,
    Repeat,
    Invalid,
}

pub struct DebuggerLoop {
    last_cmd: DebuggerCommand,
}

impl DebuggerLoop {
    pub fn new() -> DebuggerLoop {
        DebuggerLoop {
            last_cmd: DebuggerCommand::Invalid,
        }
    }

    pub fn get_user_input(&mut self) -> DebuggerCommand {
        print!("(dbg)> ");
        io::stdout().flush().expect("Failed to flush stdout");
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read user input");
        let mut cmd = parse_command(input.trim());
        if cmd == DebuggerCommand::Repeat {
            cmd = self.last_cmd
        } else {
            self.last_cmd = cmd;
        }
        cmd
    }
}

pub fn show_usage() {
    println!("Usage:");
    println!("  step (s)       - step one instruction");
    println!("  list (l)       - list instructions");
    println!("  continue (c)   - continue execution");
    println!("  <empty line>   - repeat last command");
    println!("  quit (q)       - quit debugger");
}

fn parse_command(input: &str) -> DebuggerCommand {
    let mut iter = input.split_whitespace();
    match iter.next() {
        Some("step") | Some("s") => DebuggerCommand::Step,
        Some("list") | Some("l") => DebuggerCommand::List,
        // Some("memory")  | Some("m")=> DebuggerCommand::Memory,
        Some("continue") | Some("c") => DebuggerCommand::Continue,
        Some("quit") | Some("q") => DebuggerCommand::Quit,
        None => DebuggerCommand::Repeat,
        _ => DebuggerCommand::Invalid,
    }
}
