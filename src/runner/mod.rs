use indicatif::ProgressBar;

use colored::Colorize;
use lazy_static::lazy_static;

use crate::parsing::{load_course, JsonCourseVersion, ParsingError};

use self::v1::TestRunnerV1;

mod v1;

lazy_static! {
    static ref DOTCODESCHOOL: String =
        "[ DotCodeSchool CLI ]".bold().truecolor(230, 0, 122).to_string();
    static ref OPTIONAL: String =
        "(optional)".white().dimmed().italic().to_string();
}

#[derive(Eq, PartialEq, Clone)]
pub enum TestRunnerState {
    Loaded,
    Update,
    NewSuite(usize),
    NewTest(usize, usize),
    Failed(String),
    Passed,
    Finish,
}

pub enum RunnerVersion {
    V1(TestRunnerV1),
    Undefined,
}

pub trait Runner {
    /// Advances the [Runner]'s state machine.
    ///
    /// Possible states are:
    /// - [TestRunnerState::Loaded]: initial state after JSON deserialization.
    /// - [TestRunnerState::NewSuite]: displays information about the current
    ///   suite.
    /// - [TestRunnerState::NewTest]: displays information about the current
    ///   test.
    /// - [TestRunnerState::Failed]: a mandatory test did not pass.
    /// - [TestRunnerState::Passed]: **all** mandatory tests passed.
    /// - [TestRunnerState::Finish]: finished execution.
    ///
    /// TODO: state diagram
    fn run(self) -> Self;

    /// Returns the current state of the [Runner]
    ///
    /// Possible states are:
    /// - [TestRunnerState::Loaded]: initial state after JSON deserialization.
    /// - [TestRunnerState::NewSuite]: displays information about the current
    ///   suite.
    /// - [TestRunnerState::NewTest]: displays information about the current
    ///   test.
    /// - [TestRunnerState::Failed]: a mandatory test did not pass.
    /// - [TestRunnerState::Passed]: **all** mandatory tests passed.
    /// - [TestRunnerState::Finish]: finished execution.
    fn state(&self) -> TestRunnerState;

    /// Creates a new [Runner] instance depending on the version specified in
    /// `tests.json`.
    ///
    /// * `path`: path to `tests.json`.
    fn new(path: &str) -> RunnerVersion {
        match load_course(path) {
            Ok(course_version) => match course_version {
                JsonCourseVersion::V1(course) => {
                    let test_count = course
                        .suites
                        .iter()
                        .fold(0, |acc, suite| acc + suite.tests.len());

                    let progress = ProgressBar::new(test_count as u64);

                    let runner = TestRunnerV1::new(
                        progress,
                        0,
                        TestRunnerState::Loaded,
                        course,
                    );

                    RunnerVersion::V1(runner)
                }
            },
            Err(e) => {
                let msg = match e {
                    ParsingError::CourseFmtError(msg) => msg,
                    ParsingError::FileOpenError(msg) => msg,
                };
                log::error!("{msg}");

                RunnerVersion::Undefined
            }
        }
    }
}

impl Runner for RunnerVersion {
    fn run(self) -> Self {
        match self {
            RunnerVersion::V1(runner) => Self::V1(runner.run()),
            RunnerVersion::Undefined => Self::Undefined,
        }
    }

    fn state(&self) -> TestRunnerState {
        match self {
            RunnerVersion::V1(runner) => runner.state(),
            RunnerVersion::Undefined => TestRunnerState::Finish,
        }
    }
}

/// Formats tests `stderr` and `stdout` output.
///
/// Format is as follows:
///
/// ```bash
/// ╭─[ output ]
/// │ {output}
/// ╰─[ {msg} ]
/// ```
///
/// * `output`: test output.
/// * `msg`: custom message to display after the output.
fn format_output(output: &str, msg: &str) -> String {
    let output = output.replace("\n", "\n    │");
    format!("    ╭─[ output ]{output}\n    ╰─[ {msg} ]")
}
