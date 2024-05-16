use std::{collections::HashMap, fmt::Display};

use anyhow::anyhow;
use reqwest::{
    blocking::{self, Response},
    StatusCode,
};

use crate::config::Configuration;

enum ApiEndpoint {
    BreakStart,
    BreakEnd,
    ClockIn,
    ClockOut,
}
impl ApiEndpoint {
    fn path(&self) -> String {
        match self {
            Self::BreakStart => String::from("/attendance/shifts/break_start"),
            Self::BreakEnd => String::from("/attendance/shifts/break_end"),
            Self::ClockIn => String::from("/attendance/shifts/clock_in"),
            Self::ClockOut => String::from("/attendance/shifts/clock_out"),
        }
    }
}
pub struct FactorialApi {
    client: blocking::Client,
    config: Configuration,
}
impl FactorialApi {
    pub fn new(client: blocking::Client, config: Configuration) -> FactorialApi {
        FactorialApi { client, config }
    }

    pub fn clock_in(&self, time: &str) -> anyhow::Result<()> {
        let response = self.make_api_call(ApiEndpoint::ClockIn, time)?;
        match response.status() {
            StatusCode::CREATED => Ok(()),
            StatusCode::CONFLICT => {
                println!("An open shift already exists. Doing nothing.");
                Ok(())
            }
            _ => Err(anyhow!("Could not open shift")),
        }
    }

    pub fn clock_out(&self, time: &str) -> anyhow::Result<()> {
        let response = self.make_api_call(ApiEndpoint::ClockOut, time)?;
        match response.status() {
            StatusCode::OK => Ok(()),
            _ => Err(anyhow!("Could not close shift")),
        }
    }
    pub fn break_start(&self, time: &str) -> anyhow::Result<()> {
        let response = self.make_api_call(ApiEndpoint::BreakStart, time)?;
        match response.status() {
            StatusCode::CREATED => Ok(()),
            StatusCode::CONFLICT => {
                println!("You are already taking a break. Doing nothing.");
                Ok(())
            }
            _ => Err(anyhow!("Could not start a break")),
        }
    }

    pub fn break_end(&self, time: &str) -> anyhow::Result<()> {
        let response = self.make_api_call(ApiEndpoint::BreakEnd, time)?;
        match response.status() {
            StatusCode::OK => Ok(()),
            StatusCode::CONFLICT => {
                println!("There is no ongoing break. Doing nothing.");
                Ok(())
            }
            _ => Err(anyhow!("Could not end break")),
        }
    }

    fn make_api_call(&self, endpoint: ApiEndpoint, time: &str) -> anyhow::Result<Response> {
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
