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
    fn run(&self, api: FactorialApi) -> anyhow::Result<()> {
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
            api.delete_all_shifts(start)?;
        }
        api.shift_start(start)?;
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
            api.shift_end(end)?;
        }
        Ok(())
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
    fn run(&self, api: FactorialApi) -> anyhow::Result<()> {
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
        api.shift_end(end)?;
        Ok(())
    }
}
/// take a break from an ongoing shift
#[derive(Args)]
struct BreakStart {
    /// start shift now
    #[arg(short, long, required_unless_present("time"))]
    now: bool,
    /// start shift at the specified time
    #[arg(short, long, default_value = "", conflicts_with("now"))]
    time: String,
    /// start a shift and end it after the specified duration
    #[arg(short, long, default_value = "")]
    duration: String,
    /// override existing shifts and ignore holidays and vacations
    #[arg(short, long)]
    force: bool,
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
struct Config {}
pub fn parse_args() {
    let cli = Cli::parse();
    let api = get_api();
    match cli.command {
        Commands::ShiftStart(c) => c.run(api).expect("Something went wrong"),
        Commands::ShiftEnd(c) => c.run(api).expect("Something went wrong"),
        Commands::BreakStart(_) => todo!(),
        Commands::BreakEnd(_) => todo!(),
        Commands::Auto(_) => todo!(),
        Commands::Config(_) => todo!(),
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
    if cred.get_password().is_err() {
        cred.ask_for_password().expect("Could not access keyring.")
    }

    let api = FactorialApi::new(cred, config)
        .expect("Could not authenticate to Factorial. Credentials might be wrong.");
    api
}
