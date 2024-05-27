use clap::{Args, Parser, Subcommand};

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
    /// override existing shifts and ignore holidays and vacations
    #[arg(short, long)]
    force: bool,
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
}
/// configure tracktorial
#[derive(Args)]
struct Config {}
pub fn parse_args() {
    let cli = Cli::parse();
}
