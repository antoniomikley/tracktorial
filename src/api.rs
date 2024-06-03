use std::{collections::HashMap, ops::Div};

use anyhow::anyhow;
use chrono::{DateTime, Datelike, Local};
use reqwest::{
    blocking::{self, Response},
    StatusCode,
};
use serde::Serialize;

use crate::{
    config::Configuration,
    login,
    time::{parse_date, FreeDay, HalfDay},
};

pub enum ApiEndpoint {
    BreakStart,
    BreakEnd,
    ClockIn,
    ClockOut,
    Shifts,
    Leaves,
    Holidays,
    Companies,
    Employees,
    Contracts,
    Periods,
}

impl ApiEndpoint {
    fn url(&self) -> String {
        let base_url = "https://api.factorialhr.com".to_string();
        match self {
            Self::Periods => base_url + "/attendance/periods/",
            Self::Shifts => base_url + "/attendance/shifts/",
            Self::BreakStart => base_url + "/attendance/shifts/break_start/",
            Self::BreakEnd => base_url + "/attendance/shifts/break_end/",
            Self::ClockIn => base_url + "/attendance/shifts/clock_in/",
            Self::ClockOut => base_url + "/attendance/shifts/clock_out/",
            Self::Leaves => base_url + "/leaves/",
            Self::Holidays => base_url + "/company_holidays/",
            Self::Companies => base_url + "/companies/",
            Self::Employees => base_url + "/employees/",
            Self::Contracts => base_url + "/contracts/contract_version/",
        }
    }
}

/// Provides methods to make calls to the Factorial API
pub struct FactorialApi {
    client: blocking::Client,
    config: Configuration,
}

impl FactorialApi {
    /// Takes your credentials and attempts to authenticate you to the FactorialApi.
    ///
    /// # Errors
    /// Returns an error if:
    /// - something went wrong during login, most likely due to wrong credentials
    /// - could not retrieve your employee id
    pub fn new(
        credential: login::Credential,
        mut config: Configuration,
    ) -> anyhow::Result<FactorialApi> {
        // Attempt to login to Factorial
        let client = credential.authenticate_client()?;
        if config.user_id == "" {
            // Sets factorial_data cookie which also contains the users access_id
            let response = client.get(ApiEndpoint::Companies.url()).send()?;
            let mut access_id = String::new();
            for cookie in response.cookies() {
                // Get the access_id out of the cookie
                let needle = "access_id%22%3A";
                let haystack = cookie.value();
                if haystack.contains(needle) {
                    let i = haystack.find(needle).unwrap();
                    let id = haystack.get(i + needle.len()..haystack.len()).unwrap();
                    access_id = id.get(..id.find('%').unwrap()).unwrap().to_string();
                }
            }
            // Get a list of all employees
            let response = client.get(ApiEndpoint::Employees.url()).send()?;
            let emloyees: Vec<serde_json::Value> = response.json()?;
            // Get the employee with your access_id
            for employee in emloyees {
                if employee["access_id"].to_string() == access_id {
                    config.user_id = employee["id"].to_string();
                }
            }
        }

        if config.working_hours == 0.0 {
            let response = client
                .get(ApiEndpoint::Contracts.url())
                .query(&[("employee_ids[]", &config.user_id)])
                .send()?;
            let mut contracts: Vec<serde_json::Value> = response.json()?;
            if contracts.len() == 0 {
                return Err(anyhow!("The employee has no contract. Unable to get the amount of working hours. Manually setting the amount in the configuration file can bypass this issue."));
            }
            let hours_string = contracts.pop().unwrap()["working_hours"].to_string();
            // breaks if working_hours_frequency is not "weekly"
            if hours_string.len() != 4 {
                return Err(anyhow!("The amount of working hours is either in a format that cannot be parsed or you work an unusual amount of hours a week"));
            }
            let hours_float = format!(
                "{}.{}",
                hours_string.get(0..2).unwrap(),
                hours_string.get(2..4).unwrap()
            )
            .parse::<f32>()?;
            config.working_hours = hours_float;

            // set working_week_days
            let days_string = &contracts.pop().unwrap()["working_week_days"]
                .as_str()
                .unwrap()
                .to_owned();
            let days: Vec<String> = days_string.split(',').map(|s| s.to_owned()).collect();
            config.working_week_days = days;
            config.shift_duration = config
                .working_hours
                .div(config.working_week_days.len() as f32);
        }
        config.write_config()?;
        Ok(FactorialApi { client, config })
    }

    /// Starts a shift at the given time.
    /// # Errors
    /// Returns an error if:
    /// - there already is an open shift
    /// - there is an ongoing break
    /// - there is a shift between the given time and now
    pub fn shift_start(&self, time: DateTime<Local>) -> anyhow::Result<()> {
        let response = self.post_api_call(ApiEndpoint::ClockIn, time)?;
        match response.status() {
            StatusCode::CREATED => Ok(()),
            StatusCode::CONFLICT => Err(anyhow!("An open shift already exists. Doing nothing.")),
            _ => Err(anyhow!("Could not open shift")),
        }
    }

    /// Ends a shift at the given time.
    /// # Errors
    /// Returns an error if:
    /// - there currently is no open_shift
    /// - there is a shift between the given timen and now
    pub fn shift_end(&self, time: DateTime<Local>) -> anyhow::Result<()> {
        let response = self.post_api_call(ApiEndpoint::ClockOut, time)?;
        match response.status() {
            StatusCode::OK => Ok(()),
            _ => Err(anyhow!("Could not close shift")),
        }
    }

    /// Starts a break at the given time.
    /// # Errors
    /// Returns an error if:
    /// - there already is an ongoing break
    /// - there is no open shift at that day to take a break from
    /// - there is a shift between the given timen and now
    pub fn break_start(&self, time: DateTime<Local>) -> anyhow::Result<()> {
        let response = self.post_api_call(ApiEndpoint::BreakStart, time)?;
        match response.status() {
            StatusCode::CREATED => Ok(()),
            StatusCode::CONFLICT => Err(anyhow!(
                "There already in an ongoing break or no open shift to take a break from."
            )),
            _ => Err(anyhow!("Could not start a break")),
        }
    }

    /// Ends an ongoing break at the given time.
    /// # Errors
    /// Returns an error if:
    /// - there is no ongoing break.
    /// - there is a shift between the given timen and now
    pub fn break_end(&self, time: DateTime<Local>) -> anyhow::Result<()> {
        let response = self.post_api_call(ApiEndpoint::BreakEnd, time)?;
        match response.status() {
            StatusCode::OK => Ok(()),
            _ => Err(anyhow!("Could not end break")),
        }
    }

    /// Deletes all shifts and breaks at the day of the given time and does nothing
    /// if there are no shifts or breaks.
    /// # Errors
    /// Returns an Error if the operation could not be completed.
    pub fn delete_all_shifts(&self, time: DateTime<Local>) -> anyhow::Result<()> {
        let month = time.month();
        let year = time.year();
        let day = time.day();
        let response = self
            .client
            .get(ApiEndpoint::Shifts.url())
            .query(&[
                ("employee_id", self.config.user_id.as_str()),
                ("month", &month.to_string()),
                ("year", &year.to_string()),
            ])
            .send()?;
        let shifts: Vec<serde_json::Value> = response.json()?;
        for shift in shifts {
            if shift["day"].to_string() == day.to_string() {
                let shift_id = shift["id"].to_string();
                let response = self
                    .client
                    .delete(
                        "https://api.factorialhr.com".to_string()
                            + &ApiEndpoint::Shifts.url()
                            + &shift_id,
                    )
                    .send()?;
                if response.status() != StatusCode::NO_CONTENT {
                    return Err(anyhow!("what happened"));
                }
            }
        }
        Ok(())
    }

    /// Retrieves all days on which no work has to be done. Includes holidays, paid time off and
    /// weekends.
    /// # Errors
    /// Returns an Error if:
    /// - the request could not be sent
    /// - the response body could not be parsed
    /// # Panics
    /// Panics if the received response body does not contain the required information.
    pub fn get_free_days(
        &self,
        from: DateTime<Local>,
        to: DateTime<Local>,
    ) -> anyhow::Result<Vec<FreeDay>> {
        let mut free_days: Vec<FreeDay> = Vec::new();
        let from_ymd = format!("{}", from.format("%Y-%m-%d"));
        let to_ymd = format!("{}", to.format("%Y-%m-%d"));

        let response = self.client.get(ApiEndpoint::Holidays.url()).send()?;

        let company_holidays: Vec<serde_json::Value> = response.json()?;

        for holiday in company_holidays {
            let day = parse_date(holiday["date"].as_str().unwrap()).unwrap();
            let half: HalfDay;
            if holiday["half_day"].is_null() {
                half = HalfDay::WholeDay;
            } else if holiday["half_day"].as_str().unwrap() == "end_of_day" {
                half = HalfDay::EndOfDay;
            } else {
                half = HalfDay::StartOfDay;
            }
            free_days.push(FreeDay { day, half })
        }

        let response = self
            .client
            .get(ApiEndpoint::Leaves.url())
            .query(&[
                ("employee_id", self.config.user_id.as_str()),
                ("terminated", "true"),
                ("from", from_ymd.as_str()),
                ("to", to_ymd.as_str()),
            ])
            .send()?;
        let vacations: Vec<serde_json::Value> = response.json()?;
        for vacay in vacations {
            let mut start = parse_date(vacay["start_on"].as_str().unwrap()).unwrap();
            let end = parse_date(vacay["finish_on"].as_str().unwrap()).unwrap();
            while start <= end {
                free_days.push(FreeDay {
                    day: start,
                    half: HalfDay::WholeDay,
                });
                start = start.checked_add_days(chrono::Days::new(1)).unwrap();
            }
        }

        let work_days: Vec<chrono::Weekday> = self
            .config
            .working_week_days
            .iter()
            .map(|s| s.parse::<chrono::Weekday>().unwrap())
            .collect();
        let mut start = from.clone();
        let end = to.clone();

        while start <= end {
            if !work_days.contains(&start.weekday()) {
                free_days.push(FreeDay {
                    day: start,
                    half: HalfDay::WholeDay,
                })
            }
            start = start.checked_add_days(chrono::Days::new(1)).unwrap();
        }

        Ok(free_days)
    }

    /// Creates a shift lasting from start to end
    /// # Errors
    /// Returns an Error if:
    /// - the request could not be sent
    /// - the shift could not be created, possibly because start or end overlap with an existing
    /// shift or break
    /// # Panics
    /// Panics if the period id could not be successfully retrieved.
    pub fn make_shift(
        &self,
        start: chrono::DateTime<Local>,
        end: chrono::DateTime<Local>,
    ) -> anyhow::Result<()> {
        let response = self
            .client
            .post(ApiEndpoint::Shifts.url())
            .json(&ShiftData::new(
                start,
                end,
                &self.config.location_type,
                self.get_period_id(start).unwrap(),
                false,
            ))
            .send()?;
        if response.status() != StatusCode::CREATED {
            return Err(anyhow!("Something went wrong. Shift was not created."));
        }
        Ok(())
    }

    /// Creates a break lasting from start to end
    /// # Errors
    /// Returns an Error if:
    /// - the request could not be sent
    /// - the shift could not be created, possibly because start or end overlap with an existing
    /// shift or break
    /// # Panics
    /// Panics if the period id could not be successfully retrieved.
    pub fn make_break(
        &self,
        start: chrono::DateTime<Local>,
        end: chrono::DateTime<Local>,
    ) -> anyhow::Result<()> {
        let response = self
            .client
            .post(ApiEndpoint::Shifts.url())
            .json(&ShiftData::new(
                start,
                end,
                &self.config.location_type,
                self.get_period_id(start).unwrap(),
                true,
            ))
            .send()?;
        if response.status() != StatusCode::CREATED {
            return Err(anyhow!("Something went wrong. Break was not created."));
        }
        Ok(())
    }

    /// retrieves the period id for a given date.
    /// # Errors
    /// Returns an error if the request could not be sent.
    /// # Panics
    /// Panics if the response could not be parsed.
    fn get_period_id(&self, date: chrono::DateTime<Local>) -> anyhow::Result<usize> {
        let response = self
            .client
            .get(ApiEndpoint::Periods.url())
            .query(&[
                ("year", date.year().to_string().as_str()),
                ("month", date.month().to_string().as_str()),
                ("employee_id", self.config.user_id.as_str()),
            ])
            .send()?;
        let mut periods: Vec<serde_json::Value> = response.json().unwrap();
        Ok(periods.pop().unwrap()["id"]
            .as_u64()
            .unwrap()
            .try_into()
            .unwrap())
    }

    /// simple function to make it more convenient to send a post request with a
    /// preset body.
    fn post_api_call(
        &self,
        endpoint: ApiEndpoint,
        time: DateTime<Local>,
    ) -> anyhow::Result<Response> {
        let time = time.to_rfc3339();
        let mut params = HashMap::new();
        params.insert("now".to_string(), time);
        params.insert(
            "location_type".to_string(),
            self.config.location_type.clone(),
        );
        params.insert("source".to_string(), "desktop".to_string());

        let response = self.client.post(endpoint.url()).json(&params).send()?;
        Ok(response)
    }
}

/// All the data required to create a shift or break that can be serialized to json and sent as a
/// request body.
#[derive(Serialize)]
struct ShiftData {
    clock_in: String,
    clock_out: String,
    date: String,
    day: usize,
    location_type: String,
    minutes: Option<usize>,
    period_id: usize,
    source: String,
    time_settings_break_configuration_id: Option<usize>,
    workable: bool,
}

impl ShiftData {
    fn new(
        start: chrono::DateTime<Local>,
        end: chrono::DateTime<Local>,
        location_type: &str,
        period_id: usize,
        is_break: bool,
    ) -> Self {
        ShiftData {
            clock_in: start.format("%H:%M").to_string(),
            clock_out: end.format("%H:%M").to_string(),
            date: start.format("%Y-%m-%d").to_string(),
            day: start.format("%d").to_string().parse::<usize>().unwrap(),
            location_type: location_type.to_string(),
            minutes: None,
            period_id,
            source: "desktop".to_string(),
            time_settings_break_configuration_id: None,
            workable: !is_break,
        }
    }
}
