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

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonTest {
    name: String,
    optional: bool,
    cmd: String,
    message_on_fail: String,
    message_on_success: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonTestSuite {
    name: String,
    optional: bool,
    tests: Vec<JsonTest>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonCourse {
    course: String,
    instructor: String,
    course_id: u64,
    suites: Vec<JsonTestSuite>,
}

fn load_course(path: &str) -> Result<JsonCourse, ParsingError> {
    let file_contents = std::fs::read_to_string(path)
        .map_err(|_| ParsingError::FileOpenError(path.to_string()))?;
    let json_course = serde_json::from_str::<JsonCourse>(&file_contents)
        .map_err(|err| ParsingError::CourseFmtError(err.to_string()))?;

    Ok(json_course)
}

impl JsonTest {
    pub fn execute(self) -> TestResult {
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

        match output.status.success() {
            true => TestResult::Pass(String::from_utf8(output.stdout).unwrap()),
            false => {
                TestResult::Fail(String::from_utf8(output.stderr).unwrap())
            }
        }
    }
}
