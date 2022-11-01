#[macro_use]
extern crate clap;
extern crate core;

use std::path::Path;
use std::process::exit;
use std::{env, fs};

use clap::{Parser, Subcommand};

use crate::htb::{get_machine, get_machines, join_machine, leave_machine, own_machine};
use crate::machine::{Difficulty, Machine, OperatingSystem};

mod htb;
mod machine;

const RES: &str = "\x1B[";
const HTB_MACHINE_PAGE: &str = "https://app.hackthebox.com/machines";

#[derive(Parser)]
#[command(name = "HTB")]
#[command(author = "Mitchell <13673076+Mishyy@users.noreply.github.com>")]
#[command(version)]
#[command(about = "Command-line utility for interacting with the HackTheBox API.")]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    #[command(about = "List playable machines")]
    List {
        #[arg(help = "Filter machines by operating system", long_help = None, value_enum)]
        os: Option<OperatingSystem>,
        #[arg(help = "Filter machines by difficulty", long_help = None, short, long = "diff", value_enum)]
        difficulty: Option<Difficulty>,
    },
    #[command(about = "Display information about a live machine", long_about = None)]
    Info {
        #[arg(value_parser = parse_machine)]
        machine: Machine,
        #[arg(help = "Print information in \x1B[1meval\x1B[0m-compatible format", long_help = None, long, default_value_t = false)]
        eval: bool,
        #[arg(help = "Skip creating directory", long_help = None, long, default_value_t = false)]
        no_dir: bool,
    },
    #[command(about = "Join a live machine")]
    Join {
        #[arg(value_parser = parse_machine)]
        machine: Machine,
    },
    #[command(about = "Leave the user's active machine")]
    Leave,
    #[command(about = "Submit a flag to a machine")]
    Submit {
        #[arg(value_parser = parse_machine)]
        machine: Machine,
        flag: String,
        #[arg(value_parser = clap::value_parser!(u16).range(1..=100))]
        difficulty: u16,
    },
}

fn main() {
    let args = Args::parse();
    match &args.command {
        Command::List {
            os,
            difficulty: diff,
        } => match get_machines() {
            Ok(machines) => {
                let machines = machines
                    .into_iter()
                    .filter(|m| os.unwrap_or(m.os) == m.os)
                    .filter(|m| diff.unwrap_or(m.difficulty) == m.difficulty)
                    .collect::<Vec<Machine>>();
                for machine in &machines {
                    println!(
                        "#{} {RES}1m{}{RES}0m [{:#?} <> {:#?}]",
                        machine.id, machine.name, machine.os, machine.difficulty
                    );
                }
                println!("Found {} machines.", machines.len().to_string());
            }
            Err(message) => {
                eprintln!("get_machines: {message}");
                exit(1);
            }
        },
        Command::Info {
            machine,
            eval,
            no_dir,
        } => match *eval {
            true => {
                if !*no_dir {
                    let home = Path::new(machine.home.as_str());
                    if !home.is_dir() {
                        if !home.parent().map(|p| p.exists()).unwrap_or(true) {
                            eprintln!("Are you sure {} is a valid path?", machine.home);
                            exit(2);
                        } else if home.exists() {
                            eprintln!("{} already exists but isn't a directory!", machine.home);
                            exit(2);
                        } else if let Err(error) = fs::create_dir(home) {
                            eprintln!("create_dir: {}", error.to_string());
                            exit(1);
                        }
                    }
                }

                println!("export MACHINE_ID={}", machine.id);
                println!("export MACHINE_NAME={}", machine.name);
                println!("export MACHINE_IP={}", machine.ip);
                println!(
                    "export MACHINE_HOME=\"{}\"",
                    machine.home.replace(
                        &env::var("CS_OPT").expect("You must define CS_OPT!"),
                        "$CS_OPT",
                    )
                );
            }
            false => {
                println!(
                    "#{}> {RES}1m{}{RES}0m [{:#?} <> {:#?}] @ {}",
                    machine.id, machine.name, machine.os, machine.difficulty, machine.ip
                );
                println!("{HTB_MACHINE_PAGE}/{}", machine.id);
            }
        },
        Command::Join { machine } => {
            if let Err(message) = join_machine(&machine) {
                eprintln!("spawn_machine: {message}");
                exit(1);
            }
        }
        Command::Leave => {
            if let Err(message) = leave_machine() {
                eprintln!("despawn_machine: {message}");
                exit(1);
            }
        }
        Command::Submit {
            machine,
            flag,
            difficulty,
        } => {
            if let Err(message) = own_machine(&machine, flag, *difficulty) {
                eprintln!("own_machine: {message}");
                exit(1);
            }
        }
    }
}

fn parse_machine(input: &str) -> Result<Machine, String> {
    let name = input
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect::<String>();
    return if name.is_empty() {
        Err(String::from("Invalid input after sanitation."))
    } else {
        get_machine(name.as_str())
    };
}
