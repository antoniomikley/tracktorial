use std::collections::HashMap;

use anyhow::anyhow;
use chrono::{Datelike, Local};
use reqwest::{
    blocking::{self, Response},
    StatusCode,
};

use crate::{config::Configuration, login};

pub enum ApiEndpoint {
    BreakStart,
    BreakEnd,
    ClockIn,
    ClockOut,
    Shifts,
    Leaves,
    Holidays,
}
impl ApiEndpoint {
    fn path(&self) -> String {
        match self {
            Self::BreakStart => String::from("/attendance/shifts/break_start/"),
            Self::BreakEnd => String::from("/attendance/shifts/break_end/"),
            Self::ClockIn => String::from("/attendance/shifts/clock_in/"),
            Self::ClockOut => String::from("/attendance/shifts/clock_out/"),
            Self::Shifts => String::from("/attendance/shifts/"),
            Self::Leaves => String::from("/leaves/"),
            Self::Holidays => String::from("/company_holidays/"),
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
            let response = client.get("https://api.factorialhr.com/companies").send()?;
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
            let response = client.get("https://api.factorialhr.com/employees").send()?;
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
                .get("https://api.factorialhr.com/contracts/contract_versions")
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
        }
        config.write_config()?;
        Ok(FactorialApi { client, config })
    }

    /// Starts a shift at the given time.
    /// # Errors
    /// Returns an error if:
    /// - there already is an open shift
    /// - there is an ongoing break
    /// - there is just about anything else happening at the given time
    pub fn clock_in(&self, time: &str) -> anyhow::Result<()> {
        let response = self.post_api_call(ApiEndpoint::ClockIn, time)?;
        match response.status() {
            StatusCode::CREATED => Ok(()),
            StatusCode::CONFLICT => Err(anyhow!("An open shift already exists. Doing nothing.")),
            _ => Err(anyhow!("Could not open shift")),
        }
    }

    /// Ends a shift at the given time.
    /// # Errors
    /// Returns an error if there currently is no open_shift
    pub fn clock_out(&self, time: &str) -> anyhow::Result<()> {
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
    pub fn break_start(&self, time: &str) -> anyhow::Result<()> {
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
    /// Returns an error if there is no ongoing break.
    pub fn break_end(&self, time: &str) -> anyhow::Result<()> {
        let response = self.post_api_call(ApiEndpoint::BreakEnd, time)?;
        match response.status() {
            StatusCode::OK => Ok(()),
            _ => Err(anyhow!("Could not end break")),
        }
    }

    pub fn delete_all_shifts(&self, time: chrono::DateTime<Local>) -> anyhow::Result<()> {
        let month = time.month();
        let year = time.year();
        let day = time.day();
        let response = self
            .client
            .get("https://api.factorialhr.com".to_string() + &ApiEndpoint::Shifts.path())
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
                            + &ApiEndpoint::Shifts.path()
                            + &shift_id,
                    )
                    .send()?;
                if response.status() != StatusCode::OK {
                    return Err(anyhow!(""));
                }
            }
        }
        Ok(())
    }

    fn post_api_call(&self, endpoint: ApiEndpoint, time: &str) -> anyhow::Result<Response> {
        let response = self
            .client
            .post(String::from("https://api.factorialhr.com") + &endpoint.path())
            .json(&self.make_body(time))
            .send()?;
        Ok(response)
    }

    fn make_body(&self, time: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();
        params.insert("now".to_string(), time.to_string());
        params.insert(
            "location_type".to_string(),
            self.config.location_type.clone(),
        );
        params.insert("source".to_string(), "desktop".to_string());
        params
    }
}
