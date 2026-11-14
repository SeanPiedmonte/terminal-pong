use std::process::ExitCode;

use clap::{Arg, Command};

use terminal_pong::ai::{Ai, Difficulty};
use terminal_pong::eventlog::EventLog;
use terminal_pong::{client, server, solo};

const DEFAULT_ADDR: &str = "127.0.0.1:7878";

fn cli() -> Command {
    let address = Arg::new("address").default_value(DEFAULT_ADDR).help("host:port");
    let log = Arg::new("log")
        .long("log")
        .value_name("FILE")
        .help("Append a line for every paddle hit and miss to FILE");

    Command::new("terminal-pong")
        .about("Pong in the terminal: play the computer, or a friend over the network")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("solo")
                .about("Play against the computer")
                .arg(
                    Arg::new("difficulty")
                        .short('d')
                        .long("difficulty")
                        .value_parser(Difficulty::NAMES)
                        .default_value("medium"),
                )
                .arg(log.clone()),
        )
        .subcommand(
            Command::new("server")
                .about("Host two-player matches; logs every hit and miss to stdout")
                .arg(address.clone())
                .arg(log.help("Write the hit/miss log to FILE instead of stdout")),
        )
        .subcommand(Command::new("client").about("Join a two-player match").arg(address))
}

fn main() -> ExitCode {
    let matches = cli().get_matches();
    let result = match matches.subcommand() {
        Some(("solo", args)) => {
            let difficulty = Difficulty::parse(args.get_one::<String>("difficulty").unwrap())
                .expect("clap only accepts known difficulties");
            match args.get_one::<String>("log").map(|path| EventLog::file(path)).transpose() {
                Ok(log) => solo::run(Ai::new(difficulty), log),
                Err(err) => Err(err),
            }
        }
        Some(("server", args)) => {
            let addr = args.get_one::<String>("address").unwrap();
            match args.get_one::<String>("log") {
                Some(path) => EventLog::file(path),
                None => Ok(EventLog::stdout()),
            }
            .and_then(|log| server::run(addr, log))
        }
        Some(("client", args)) => {
            let addr = args.get_one::<String>("address").unwrap();
            client::run(addr).map_err(|err| {
                if err.kind() == std::io::ErrorKind::ConnectionRefused {
                    std::io::Error::new(err.kind(), format!("no server at {addr}. Start one with: terminal-pong server {addr}"))
                } else {
                    err
                }
            })
        }
        _ => unreachable!("a subcommand is required"),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("terminal-pong: {err}");
            ExitCode::FAILURE
        }
    }
}
