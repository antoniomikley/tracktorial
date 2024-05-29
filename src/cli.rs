use std::{fmt::write, io::stderr, process::exit};

use chrono::{DateTime, Local};
use clap::{Args, Parser, Subcommand};

use crate::{api::FactorialApi, config::Configuration, login::Credential, time};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}
#[derive(Subcommand)]
enum Commands {
    ShiftStart(ShiftStart),
    ShiftEnd(ShiftEnd),
    BreakStart(BreakEnd),
    BreakEnd(BreakEnd),
    Auto(Auto),
    Config(Config),
}

/// start a new shift
#[derive(Args)]
struct ShiftStart {
    /// start shift now
    #[arg(short, long, required_unless_present("time"))]
    now: bool,
    /// start shift at the specified time
    #[arg(short, long, default_value = "", conflicts_with("now"))]
    time: String,
    /// start a shift either now or at <TIME> and end it after <DURATION> or its default value
    #[arg(short, long, default_value = "")]
    duration: String,
    /// the started shift should end at <END>
    #[arg(
        short,
        long,
        default_value = "",
        requires("time"),
        conflicts_with("duration")
    )]
    end: String,
    /// override existing shifts and ignore holidays and vacations
    #[arg(short, long)]
    force: bool,
}
impl ShiftStart {
    fn run(&self, api: FactorialApi) {
        let start: DateTime<Local>;
        if self.now == true {
            start = Local::now();
        } else {
            start = match time::parse_date_time(&self.time) {
                Ok(t) => t,
                Err(_) => {
                    eprintln!("Could not parse time. Time has to be either in the format of 'year-month-dayThour:minute:second', 'hour:minute:second', 'hour:minute', or 'hour'");
                    exit(0)
                }
            }
        }
        if self.force == true {
            api.delete_all_shifts(start).expect(
                "There should not really be a case where this happens. Could be wrong tho.",
            );
        }
        if api.shift_start(start).is_err() {
            eprintln!("Could not start shift. Either there is already an open shift, you are on break, or cosmic rays have flipped a byte in your device or the Factorial server.");
            exit(0)
        }
        if self.duration.len() != 0 || self.end.len() != 0 {
            let end: DateTime<Local>;
            if self.duration.len() != 0 {
                let duration = match time::parse_duration(&self.duration) {
                    Ok(d) => d,
                    Err(_) => {
                        eprintln!("Could not parse duration. duration has to be in the format of for example '14h30m11s', '14h30m', '14h', '30m', '11s'.");
                        exit(0)
                    }
                };
                end = start + duration;
            } else {
                end = match time::parse_date_time(&self.end) {
                    Ok(t) => t,
                    Err(_) => {
                        eprintln!("Could not parse time. Time has to be either in the format of 'year-month-dayThour:minute:second', 'hour:minute:second', 'hour:minute', or 'hour'");
                        exit(0)
                    }
                }
            }
            api.shift_end(end).expect("Error handling is hard and I am not shure I am doing it right. This message should never show.");
        }
    }
}
/// end an ongoing shift
#[derive(Args)]
struct ShiftEnd {
    /// end shift now
    #[arg(short, long, required_unless_present("time"))]
    now: bool,
    /// end shift at the specified time
    #[arg(short, long, default_value = "", conflicts_with("now"))]
    time: String,
}
impl ShiftEnd {
    fn run(&self, api: FactorialApi) {
        let end: DateTime<Local>;
        if self.now == true {
            end = Local::now();
        } else {
            end = match time::parse_date_time(&self.time) {
                Ok(t) => t,
                Err(_) => {
                    eprintln!("Could not parse time. Time has to be either in the format of 'year-month-dayThour:minute:second', 'hour:minute:second', 'hour:minute', or 'hour'");
                    exit(0)
                }
            }
        }
        if api.shift_end(end).is_err() {
            eprintln!("There is no open shift.");
            exit(0)
        }
    }
}
/// take a break from an ongoing shift
#[derive(Args)]
struct BreakStart {
    /// start a break now
    #[arg(short, long, required_unless_present("time"))]
    now: bool,
    /// start a break at the specified time
    #[arg(short, long, default_value = "", conflicts_with("now"))]
    time: String,
    /// start a break and end it after the specified duration
    #[arg(short, long, default_value = "")]
    duration: String,
    /// the started shift should end at <END>
    #[arg(
        short,
        long,
        default_value = "",
        requires("time"),
        conflicts_with("duration")
    )]
    end: String,
}
impl BreakStart {
    fn run(&self, api: FactorialApi) {
        let start: DateTime<Local>;
        if self.now == true {
            start = Local::now();
        } else {
            start = match time::parse_date_time(&self.time) {
                Ok(t) => t,
                Err(_) => {
                    eprintln!("Could not parse time. Time has to be either in the format of 'year-month-dayThour:minute:second', 'hour:minute:second', 'hour:minute', or 'hour'");
                    exit(0)
                }
            }
        }
        if api.break_start(start).is_err() {
            eprintln!("Something went wrong. There is either already an ongoing break or there is no open shift to take a break from.");
            exit(0)
        }
        if self.duration.len() != 0 || self.end.len() != 0 {
            let end: DateTime<Local>;
            if self.duration.len() != 0 {
                let duration = match time::parse_duration(&self.duration) {
                    Ok(d) => d,
                    Err(_) => {
                        eprintln!("Could not parse duration. duration has to be in the format of for example '14h30m11s', '14h30m', '14h', '30m', '11s'.");
                        exit(0)
                    }
                };
                end = start + duration;
            } else {
                end = match time::parse_date_time(&self.end) {
                    Ok(t) => t,
                    Err(_) => {
                        eprintln!("Could not parse time. Time has to be either in the format of 'year-month-dayThour:minute:second', 'hour:minute:second', 'hour:minute', or 'hour'");
                        exit(0)
                    }
                }
            }
            api.break_end(end).expect(
                "This should never happen. Things should have went to shit way before this.",
            );
        }
    }
}
/// end an ongoing break
#[derive(Args)]
struct BreakEnd {
    /// end break now
    #[arg(short, long, required_unless_present("time"))]
    now: bool,
    /// end break at the specified time
    #[arg(short, long, default_value = "", conflicts_with("now"))]
    time: String,
}
impl BreakEnd {
    fn run(&self, api: FactorialApi) {
        let end: DateTime<Local>;
        if self.now == true {
            end = Local::now();
        } else {
            end = match time::parse_date_time(&self.time) {
                Ok(t) => t,
                Err(_) => {
                    eprintln!("Could not parse time. Time has to be either in the format of 'year-month-dayThour:minute:second', 'hour:minute:second', 'hour:minute', or 'hour'");
                    exit(0)
                }
            }
        }
        if api.break_end(end).is_err() {
            eprintln!("There is no ongoing break.");
            exit(0)
        }
    }
}
/// manage shifts and breaks automatically
#[derive(Args)]
struct Auto {
    /// start a shift now or at <START> if given, also takes an appropriately sized break
    #[arg(short, long, conflicts_with("stop"), default_value = "")]
    duration: String,
    /// start a shift at <START> until <STOP> or with a given <DURATION>. If neither is present
    /// the default value is used.
    #[arg(long, default_value = "")]
    start: String,
    /// if <START> is given, start a shift lasting until <STOP>. mutually exclusive with
    /// <DURATION>
    #[arg(long, default_value = "")]
    stop: String,
    #[arg(long, requires("to"))]
    /// start a shift everyday starting at <FROM> and until <TO> using either <START> and <STOP> or <DURATION> or the default value.
    from: String,
    /// requires <FROM>
    #[arg(long)]
    to: String,
    /// override existing shifts, ignore holidays and vacations
    #[arg(short, long)]
    force: bool,
    /// add a random offset to all time related values
    #[arg(short, long)]
    randomize: bool,
}
/// configure tracktorial
#[derive(Args)]
struct Config {
    /// set your email address
    #[arg(short, long, default_value = "", default_missing_value = "interactive")]
    email: String,
    /// reset your password
    #[arg(short, long)]
    reset_password: bool,
}
impl Config {
    fn run(&self) {
        let mut config = match Configuration::get_config() {
            Ok(config) => config,
            Err(_) => {
                eprintln!("Could not retrieve the contents of the configuration file. Either it does not exist or the contents are invalid.");
                exit(0)
            }
        };

        if self.email == "interactive" {
            config.prompt_for_email().unwrap();
            exit(0)
        }
        if self.email != "" {
            config.email = self.email.clone();
        }
        if self.reset_password {
            let mut cred = Credential::new_without_password(&config.email);
            cred.reset_password().unwrap();
        }
    }
}
pub fn parse_args() {
    let cli = Cli::parse();
    match cli.command {
        Commands::ShiftStart(c) => c.run(get_api()),
        Commands::ShiftEnd(c) => c.run(get_api()),
        Commands::BreakStart(c) => c.run(get_api()),
        Commands::BreakEnd(c) => c.run(get_api()),
        Commands::Auto(_) => todo!(),
        Commands::Config(c) => c.run(),
    }
}

fn get_api() -> FactorialApi {
    let mut config = Configuration::get_config()
        .expect("Could either not create or read the configuration file.");
    if config.email.len() == 0 {
        config
            .prompt_for_email()
            .expect("Could either not read email from stdin or save it to the configuration file");
    }

    let mut cred = Credential::new_without_password(&config.email);

    for _ in 0..3 {
        if cred.get_password().is_err() {
            cred.ask_for_password().expect("Could not access keyring.")
        }

        let _api = match FactorialApi::new(cred.clone(), config.clone()) {
            Ok(api) => return api,
            Err(_) => {
                eprintln!("Could not login to Factorial. Credentials might be wrong.");
                cred.reset_password()
            }
        };
    }
    exit(0)
}
