use std::ops::Deref;

use indicatif::ProgressBar;

use colored::{ColoredString, Colorize};
use lazy_static::lazy_static;

use crate::parsing::{load_course, JsonCourse, ParsingError, TestResult};

const V_1_0: &str = "1.0";

lazy_static! {
    static ref OPTIONAL: ColoredString = "(optional)".white().dimmed().italic();
}

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
    success: u32,
    pub state: TestRunnerState,
    course: JsonCourse,
}

#[derive(Eq, PartialEq)]
pub enum TestRunnerState {
    Loaded,
    NewSuite(usize),
    NewTest(usize, usize),
    Failed(String),
    Passed,
    Finish,
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

                TestRunner {
                    progress,
                    success: 0,
                    state: TestRunnerState::Loaded,
                    course,
                }
            }
            Err(e) => {
                let msg = match e {
                    ParsingError::CourseFmtError(msg) => msg,
                    ParsingError::FileOpenError(msg) => msg,
                };
                log::error!("{msg}");

                // TODO: deserialization should happen AFTER the version has
                // been determined
                let mut course = JsonCourse::default();
                course.version = V_1_0.to_string();

                TestRunner {
                    progress: ProgressBar::new(0),
                    success: 0,
                    state: TestRunnerState::Failed(
                        format!("could not parse test file").to_string(),
                    ),
                    course,
                }
            }
        }
    }

    pub fn run(self) -> Self {
        let Self { progress, success, state, course } = self;

        match course.version.deref() {
            V_1_0 => match state {
                TestRunnerState::Loaded => {
                    progress.println(format!(
                        "{}",
                        "[ DotCodeSchool CLI ]".bold().truecolor(230, 0, 122)
                    ));

                    progress.println(format!(
                        "{} by {}",
                        course.name.to_uppercase().white().bold(),
                        course.instructor.white().bold()
                    ));

                    let exercise_count = course
                        .suites
                        .iter()
                        .fold(0, |acc, suite| acc + suite.tests.len());
                    progress.println(format!(
                        "\n📒 You have {} exercises left",
                        exercise_count
                    ));

                    Self {
                        progress,
                        success,
                        state: TestRunnerState::NewSuite(0),
                        course,
                    }
                }
                TestRunnerState::NewSuite(index_suite) => {
                    let suite = &course.suites[index_suite];
                    let suite_str =
                        suite.name.deref().to_uppercase().bold().green();

                    progress.println(format!(
                        "\n{suite_str} {}",
                        if suite.optional {
                            OPTIONAL.clone()
                        } else {
                            ColoredString::default()
                        },
                    ));

                    Self {
                        progress,
                        success,
                        state: TestRunnerState::NewTest(index_suite, 0),
                        course,
                    }
                }
                TestRunnerState::NewTest(index_suite, index_test) => {
                    let suite = &course.suites[index_suite];
                    let test = &suite.tests[index_test];
                    let test_name = test.name.to_lowercase().bold();

                    progress.println(format!(
                        "\n  🧪 Running test {test_name} {}",
                        if test.optional {
                            OPTIONAL.clone()
                        } else {
                            ColoredString::default()
                        },
                    ));

                    let success_increment = match test.run() {
                        TestResult::Pass(stdout) => {
                            progress.println(Self::format_output(
                                &stdout,
                                &format!("✅ {}", &test.message_on_success),
                            ));

                            1
                        }
                        TestResult::Fail(stderr) => {
                            progress.println(format!(
                                "{}",
                                Self::format_output(
                                    &stderr,
                                    &format!("❌ {}", &test.message_on_fail)
                                )
                                .red()
                                .dimmed()
                            ));
                            if !test.optional && !suite.optional {
                                return Self {
                                    progress,
                                    success,
                                    state: TestRunnerState::Failed(format!(
                                        "Failed test {test_name}"
                                    )),
                                    course,
                                };
                            }
                            0
                        }
                    };
                    progress.inc(1);

                    match (
                        index_suite + 1 < course.suites.len(),
                        index_test + 1 < suite.tests.len(),
                    ) {
                        (_, true) => Self {
                            progress,
                            success: success + success_increment,
                            state: TestRunnerState::NewTest(
                                index_suite,
                                index_test + 1,
                            ),
                            course,
                        },
                        (true, false) => Self {
                            progress,
                            success: success + success_increment,
                            state: TestRunnerState::NewSuite(index_suite + 1),
                            course,
                        },
                        (false, false) => Self {
                            progress,
                            success: success + success_increment,
                            state: TestRunnerState::Passed,
                            course,
                        },
                    }
                }
                TestRunnerState::Failed(msg) => {
                    progress.finish_and_clear();
                    progress
                        .println(format!("\n⚠ Error: {}", msg.red().bold()));

                    Self {
                        progress,
                        success,
                        state: TestRunnerState::Finish,
                        course,
                    }
                }
                TestRunnerState::Passed => {
                    progress.finish_and_clear();
                    let exercise_count = course
                        .suites
                        .iter()
                        .fold(0, |acc, suite| acc + suite.tests.len());
                    let score = format!(
                        "{:.2}",
                        success as f64 / exercise_count as f64 * 100f64
                    );

                    progress.println(format!(
                        "\n🏁 final score: {}%",
                        score.green().bold()
                    ));

                    Self {
                        progress,
                        success,
                        state: TestRunnerState::Finish,
                        course,
                    }
                }
                TestRunnerState::Finish => Self {
                    progress,
                    success,
                    state: TestRunnerState::Finish,
                    course,
                },
            },
            _ => {
                progress.println(format!(
                    "⚠ Unsupported version {}",
                    course.version.red().bold()
                ));

                progress.finish();

                Self {
                    progress,
                    success,
                    state: TestRunnerState::Finish,
                    course,
                }
            }
        }
    }

    fn format_output(output: &str, msg: &str) -> String {
        let output = output.replace("\n", "\n    │");
        format!("    ╭─[ output ]{output}\n    ╰─[ {msg} ]")
    }
}
