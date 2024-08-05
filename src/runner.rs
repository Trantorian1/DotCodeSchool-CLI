use indicatif::ProgressBar;

use crate::parsing::{load_course, JsonCourse, ParsingError};

/// Runs all the tests specified in a `tests.json` file.
///
/// Tests are run sequentially in their order of definition. Running tests
/// occurs in 3 steps:
///
/// 1. Loading the `tests.json` file.
/// 2. Executing tests one by one, displaying `stderr` and `stdout` as
///    appropriate.
/// 3. Test stop running once all test have been run or a mandatory test fails.
/// 4. A summary of the run is displayed at the end of the process.
///
/// # `tests.json` file format
///
/// ## Version 1.0
///
/// Capabilities are divided into 3 parts:
///
/// - Course definition.
/// - Test suite definition
/// - Test definition
///
/// ### Course definition
///
/// ```json
/// {
///     "version": "1.0",
///     "course": "Course name",
///     "instructor": "Instructor name",
///     "course_id": 123,
///     "suites": [
///         ...
///     ]
/// }
/// ```
///
/// Course Id will be checked against the DotCodeScool servers to make sure that
/// the tests are being run in the correct git repository.
///
/// ### Suite definition
///
/// ```json
/// {
///     "name": "Suite name",
///     "optional": false,
///     "tests": [
///         ...
///     ]
/// }
/// ```
///
/// Test suites marked as optional do not need to be passed for the course to be
/// validated. They will however still count towards the overall success of the
/// course, so if a student passes 9 mandatory test suites but fails 1 optional
/// test suite, their overall score will still be 90%.
///
/// ### Test definition
///
/// ```json
/// {
///     "name": "Test name",
///     "optional": false,
///     "cmd": "cargo test test_name",
///     "message_on_fail": "This test failed, back to the drawing board.",
///     "message_on_success": "This test passed, congrats!"
/// }
/// ```
///
/// `cmd` defines which command to run for the test to execute. Like test
/// suites, tests can be marked as `optional`. `optional` tests will still count
/// towards the overall success of the course but do not need to be validated as
/// part of a test suite.
///
/// * `progress`: number of tests left to run.
/// * `course`: deserialized course information.
pub struct TestRunner {
    progress: ProgressBar,
    state: TestRunnerState,
    course: JsonCourse,
}

pub enum TestRunnerState {
    Loaded,
    Running,
    Finished,
}

impl TestRunner {
    pub fn new(path: &str) -> Self {
        match load_course(path) {
            Ok(course) => {
                let test_count = course
                    .suites
                    .iter()
                    .fold(0, |acc, suite| acc + suite.tests.len());

                let progress = ProgressBar::new(test_count as u64);

                TestRunner { progress, state: TestRunnerState::Loaded, course }
            }
            Err(e) => {
                let msg = match e {
                    ParsingError::CourseFmtError(msg) => msg,
                    ParsingError::FileOpenError(msg) => msg,
                };
                log::error!("{msg}");

                TestRunner {
                    progress: ProgressBar::new(0),
                    state: TestRunnerState::Finished,
                    course: JsonCourse::default(),
                }
            }
        }
    }

    fn make_progress(&mut self) -> &Self {
        match self.state {
            TestRunnerState::Loaded => todo!(),
            TestRunnerState::Running => todo!(),
            TestRunnerState::Finished => self,
        }
    }
}
