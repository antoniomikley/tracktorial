use crate::{api::FactorialApi, config::Configuration, login::Credential, time};
use chrono::{DateTime, Local, NaiveTime};
use clap::{Args, Parser, Subcommand};
use std::{process::exit, u16};

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
    BreakStart(BreakStart),
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
                    eprintln!("{}", TIME_ERR_MSG);
                    exit(0)
                }
            }
        }
        if self.force == true {
            match api.delete_all_shifts(start) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("{}", e.to_string());
                    exit(0)
                }
            };
        }
        match api.shift_start(start) {
            Err(e) => {
                eprintln!("{}", e.to_string());
                exit(0)
            }
            Ok(_) => {}
        }
        if self.duration.len() != 0 || self.end.len() != 0 {
            let end: DateTime<Local>;
            if self.duration.len() != 0 {
                let duration = match time::parse_duration(&self.duration) {
                    Ok(d) => d,
                    Err(_) => {
                        eprintln!("{}", DUR_ERR_MSG);
                        exit(0)
                    }
                };
                end = start + duration;
            } else {
                end = match time::parse_date_time(&self.end) {
                    Ok(t) => t,
                    Err(_) => {
                        eprintln!("{}", TIME_ERR_MSG);
                        exit(0)
                    }
                }
            }
            match api.shift_end(end) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("{}", e.to_string());
                    exit(0)
                }
            };
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
                    eprintln!("{}", TIME_ERR_MSG);
                    exit(0)
                }
            }
        }
        api.shift_end(end).unwrap_or_else(|err| {
            eprintln!("{}", err.to_string());
            exit(0)
        })
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
                    eprintln!("{}", TIME_ERR_MSG);
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
                        eprintln!("{}", DUR_ERR_MSG);
                        exit(0)
                    }
                };
                end = start + duration;
            } else {
                end = match time::parse_date_time(&self.end) {
                    Ok(t) => t,
                    Err(_) => {
                        eprintln!("{}", TIME_ERR_MSG);
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
                    eprintln!("{}", TIME_ERR_MSG);
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
    /// start to work now, take a break, go home. Uses the default duration if
    /// <DURATION> or <END> is not given
    #[arg(short, long, conflicts_with("start"), required_unless_present("start"))]
    now: bool,
    /// start a shift now if <NOW> is set or at <START> with the given duration, also takes an appropriately sized break.
    #[arg(short, long, conflicts_with("end"), default_value = "")]
    duration: String,
    /// start a shift at <START> until <END> or with a given <DURATION>. If neither is present
    /// the default duration is used
    #[arg(long, default_value = "")]
    start: String,
    /// if <START> is given, start a shift lasting until <STOP>. mutually exclusive with
    /// <DURATION>
    #[arg(long, default_value = "")]
    end: String,
    #[arg(long, requires("to"), default_value = "")]
    /// start a shift everyday starting at <FROM> and until <TO> using either <START> and <STOP> or <DURATION> or the default value.
    from: String,
    /// requires <FROM>
    #[arg(long, requires("from"), default_value = "")]
    to: String,
    /// override existing shifts, ignore holidays and vacations
    #[arg(short, long)]
    force: bool,
    /// add a random offset to all time related values
    #[arg(short, long)]
    randomize: bool,
}

impl Auto {
    fn run(&self, api: FactorialApi) {
        let mut start: chrono::DateTime<Local>;
        let duration: chrono::Duration;
        let mut from: chrono::DateTime<Local>;
        let to: chrono::DateTime<Local>;
        let config = &api.config;

        if self.start != "" {
            start = match time::parse_date_time(&self.start) {
                Ok(t) => t,
                Err(_) => {
                    eprintln!("{}", TIME_ERR_MSG);
                    exit(0)
                }
            };
        } else {
            start = Local::now();
        }

        if self.duration != "" {
            duration = match time::parse_duration(&self.duration) {
                Ok(d) => d,
                Err(_) => {
                    eprintln!("{}", DUR_ERR_MSG);
                    exit(0)
                }
            }
        } else if self.end != "" {
            let end_date = match time::parse_date_time(&self.end) {
                Ok(t) => t,
                Err(_) => {
                    eprintln!("{}", TIME_ERR_MSG);
                    exit(0)
                }
            };
            duration = end_date.signed_duration_since(start);
        } else {
            let dur_secs = config.shift_duration * 60.0 * 60.0;
            duration = chrono::Duration::seconds(dur_secs.floor() as i64);
        }

        if self.from != "" {
            from = match time::parse_date(&self.from) {
                Ok(t) => t,
                Err(_) => {
                    eprintln!("{}", TIME_ERR_MSG);
                    exit(0)
                }
            };
            to = match time::parse_date(&self.to) {
                Ok(t) => t,
                Err(_) => {
                    eprintln!("{}", TIME_ERR_MSG);
                    exit(0)
                }
            };
        } else {
            let now = Local::now()
                .with_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap())
                .unwrap();
            from = now;
            to = now;
        }

        let mut free_days = api.get_free_days(from, to).unwrap();
        free_days.sort_unstable();

        while from <= to {
            let needle = free_days.binary_search(&from.into());
            match needle {
                Ok(num) => {
                    if free_days.get(num).unwrap().half == time::HalfDay::WholeDay {
                        from = from.checked_add_days(chrono::Days::new(1)).unwrap();
                        continue;
                    }
                }
                Err(_) => {}
            }
            if self.force {
                api.delete_all_shifts(from).unwrap();
            }
            let work_day: time::WorkDay;
            start = from.with_time(start.time()).unwrap();
            if self.randomize {
                work_day = time::WorkDay::randomize_shift(start, duration, config.max_rand_range);
            } else {
                work_day = time::WorkDay::standard_shift(start, duration);
            }

            api.make_shift(work_day.clock_in, work_day.break_start)
                .unwrap();
            api.make_break(work_day.break_start, work_day.break_end)
                .unwrap();
            api.make_shift(work_day.break_end, work_day.clock_out)
                .unwrap();
            from = from.checked_add_days(chrono::Days::new(1)).unwrap();
        }
    }
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
    /// set the maximum amount of deviation in minutes from specified times and durations when the
    /// randomization option is enabled.
    #[arg(long, default_value = "16")]
    rand_range: String,
}
impl Config {
    fn run(&self) {
        let mut config = Configuration::get_config().expect("Could not retrieve configuration. Either the file does not exist or the user has no persmissions to access it.");
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
        if self.rand_range != "16" {
            config.max_rand_range = match self.rand_range.parse::<u16>() {
                Ok(num) => num,
                Err(_) => {
                    eprintln!("rand_range has to be a valid number between 0 and 120");
                    exit(0)
                }
            };
            if config.max_rand_range > 120 {
                eprintln!("rand_range has to be a valid number between 0 and 120");
                exit(0)
            }
        }
    }
}

pub fn parse_args() {
    let cli = Cli::parse();
    let err = |error: anyhow::Error| {
        eprintln!("{}", error.to_string());
        exit(0)
    };
    match cli.command {
        Commands::ShiftStart(c) => c.run(FactorialApi::get_api().unwrap_or_else(err)),
        Commands::ShiftEnd(c) => c.run(FactorialApi::get_api().unwrap_or_else(err)),
        Commands::BreakStart(c) => c.run(FactorialApi::get_api().unwrap_or_else(err)),
        Commands::BreakEnd(c) => c.run(FactorialApi::get_api().unwrap_or_else(err)),
        Commands::Auto(c) => c.run(FactorialApi::get_api().unwrap_or_else(err)),
        Commands::Config(c) => c.run(),
    }
}

const DUR_ERR_MSG: &str = "Could not parse duration. Duration has to be in the format of for example '14h30m11s', '14h30m', '14h', '30m', '11s'.";
const TIME_ERR_MSG: &str = "Could not parse time. Time has to be either in the format of 'year-month-dayThour:minute:second', 'hour:minute:second', 'hour:minute', or 'hour'";
