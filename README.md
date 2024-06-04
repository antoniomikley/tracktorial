# Tracktorial
- [Usage](##usage)
    - [Examples](###examples)
- [Installation](##installation)
    - [Building from source](###building-from-source)
- [Configuration](##Configuration)
    - [Configuration options](###configuration-options)

## Usage
Tracktorial can be run by executing the executable from the terminal.
```
tracktorial <COMMAND> [OPTIONS]

Commands:
    shift-start
      -n, --now                      Start shift now
      -t, --time <TIME>              Start shift at the specified time 
      -d, --duration <DURATION>      Start a shift either now or at <TIME> and end it after <DURATION> or its default value 
      -e, --end <END>                The started shift should end at <END>
      -f, --force                    Override existing shifts
      -h, --help                     Print help
    shift-end
      -n, --now                      End shift now
      -t, --time <TIME>              End shift at the specified time 
      -h, --help                     Print help
    break-start
      -n, --now                      Start a break now
      -t, --time <TIME>              Start a break at the specified time
      -d, --duration <DURATION>      Start a break and end it after the specified duration
      -e, --end <END>                The started shift should end at <END> 
      -h, --help                     Print help
    break-end
      -n, --now                      End break now
      -t, --time <TIME>              End break at the specified time 
      -h, --help                     Print help
    auto
      -n, --now                      Start to work now, take a break, go home. Uses the default duration if <DURATION> or <END> is not given
      -d, --duration <DURATION>      Start a shift now if <NOW> is set or at <START> with the given duration, also takes an appropriately sized break 
          --start <START>            Start a shift at <START> until <END> or with a given <DURATION>. If neither is present the default duration is used 
          --end <END>                If <START> is given, start a shift lasting until <STOP>. mutually exclusive with <DURATION> 
          --from <FROM>              Start a shift everyday starting at <FROM> and until <TO> using either <START> and <STOP> or <DURATION> or the default value 
          --to <TO>                  Requires <FROM> 
      -f, --force                    Override existing shifts
      -r, --randomize                Add a random offset to all time related values
      -h, --help                     Print help
    config
      -e, --email <EMAIL>            Set your email address 
      -r, --reset-password           Reset your password
          --rand-range <RAND_RANGE>  Set the maximum amount of deviation in minutes from specified times and durations when the randomization option is enabled [default: 16]
      -h, --help                     Print help
```
### Examples
Start a shift now:
```
tracktorial shift-start --now
```
Start a break at 12:30:
```
tracktorial break-start --time 12:30
```
Continue working at 13:00 and work for 4 hours and 30 minutes before clocking out:
```
tracktorial shift-start --time 13:00 --duration 4h30m
```
Start working at 7:30, work for 8 hours and take a break somewhere inbetween:
```
tracktorial auto --start 7:30 --duration 8h
```
Start working everyday at 8:00 for 8 hours from the 1st of May to the 31st of May 2024 and override existing shifts in this timeframe, also apply a random offset as to not seem to suspiciously consistent:
```
tracktorial auto --start 8:00 --duration 8h --force --randomize
```

## Installation
### Building from source
1. Install the [rust toolchain](https://rustup.rs/)
2. Clone the repo
```
git clone https://github.com/antoniomikley/tracktorial
```
3. Change to the repo's directory
```
cd tracktorial
```
4. Build the application
```
cargo build --release
```
5. This will produce an executable at `target/release/tracktorial`

Tests can be run with `cargo test` and documentation can be built with `cargo doc`.
## Configuration
Tracktorial can be configured through the CLI or via the configuration file.
Depending on your operating system the configuration file can be found in a different location:
- Linux: `$HOME/.config/tracktorial/config.json`
- macOS: '$HOME/Library/Application Support/Tracktorial/config.json'
- Windows: `%AppData%\Local\Tracktorial\config.json`

The configuration file has to be valid json, meaning comments are currently not supported.
All options are beeing populated when the program first runs and the user enters an e-mail address and password,
but a minimal working example could look like this:
```
{
  "email": "example@domain.com"
}
```
A complete example could look like this:
```
{
  "email": "example@domain.com",
  "location_type": "office",
  "user_id": "1231234",
  "working_hours": 40.0,
  "working_week_days": [
    "monday",
    "tuesday",
    "wednesday",
    "thursday",
    "friday"
  ],
  "shift_duration": 8.0,
  "max_rand_range": 30
}
```

An invalid configuration can cause the application to panic. Should an update introduce
breaking changes to the configuration file or the user made changes resulting in a broken
config, then reducing the configuration to the minimal working example could fix the probelm.
Deleting the file can achieve similar results, but will require the user to log in again.

### Configuration options
- `email`: The user's e-mail address
- `location_type`: Can be either "office" or "work_from_home"
- `user_id`: The users ID in Factorial. There is no need to manually enter or modify this value
- `working_hours`: The amount of hours the user is contracted to work in a week, if set to 0.0 tracktorial will query factorial and populate this value
- `working_week_days`: The days of the week the user is contracted to work on, tracktorial populates this value automatically
- `shift_duration`: The amount of hours to work everyday when using the `auto` sub command. Defaults to working_hours divided by the length of working_week_days
- `max_rand_range`: The random offset applied to your clock in time n minutes when using the `auto` sub command in conjuction with the `--randomize` flag

