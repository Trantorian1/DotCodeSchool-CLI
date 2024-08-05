//! A module for parsing `tests.json` files.
//!
//! This module is concerned with loading `test.json` files, parsing them and
//! executing providing an implementation for executing tests. The actual
//! execution is the responsibility of the test [runner].

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParsingError {
    #[error("failed to open course file at {0}")]
    FileOpenError(String),
    #[error("")]
    CourseFmtError(String),
}

pub enum TestResult {
    Pass(String),
    Fail(String),
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct JsonTest {
    pub name: String,
    pub optional: bool,
    pub cmd: String,
    pub message_on_fail: String,
    pub message_on_success: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct JsonTestSuite {
    pub name: String,
    pub optional: bool,
    pub tests: Vec<JsonTest>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct JsonCourse {
    pub version: String,
    #[serde(rename = "course")]
    pub name: String,
    pub instructor: String,
    pub course_id: u64,
    pub suites: Vec<JsonTestSuite>,
}

pub fn load_course(path: &str) -> Result<JsonCourse, ParsingError> {
    log::debug!("Loading course '{path}'");

    let file_contents = std::fs::read_to_string(path)
        .map_err(|_| ParsingError::FileOpenError(path.to_string()))?;
    let json_course = serde_json::from_str::<JsonCourse>(&file_contents)
        .map_err(|err| ParsingError::CourseFmtError(err.to_string()))?;

    log::debug!("Course loaded successfully!");

    Ok(json_course)
}

impl JsonTest {
    pub fn execute(self) -> TestResult {
        log::debug!("Running test: '{}'", self.cmd);
        let command: Vec<&str> = self.cmd.split_whitespace().collect();

        let output = std::process::Command::new(command[0])
            .args(command[1..].into_iter())
            .output();
        let output = match output {
            Ok(output) => output,
            Err(_) => {
                return TestResult::Fail("could not execute test".to_string())
            }
        };

        log::debug!("Test executed successfully!");

        match output.status.success() {
            true => TestResult::Pass(String::from_utf8(output.stdout).unwrap()),
            false => {
                TestResult::Fail(String::from_utf8(output.stderr).unwrap())
            }
        }
    }
}
