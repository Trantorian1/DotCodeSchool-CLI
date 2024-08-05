use std::ops::Deref;

use indicatif::ProgressBar;

use colored::Colorize;
use lazy_static::lazy_static;

use crate::parsing::{load_course, JsonCourse, ParsingError, TestResult};

const V_1_0: &str = "1.0";

lazy_static! {
    static ref OPTIONAL: String =
        "(optional)".white().dimmed().italic().to_string();
    static ref DOTCODESCHOOL: String =
        "[ DotCodeSchool CLI ]".bold().truecolor(230, 0, 122).to_string();
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

    /// Advances the [TestRunner]'s state machine.
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
    pub fn run(self) -> Self {
        let Self { progress, mut success, state, course } = self;

        match course.version.deref() {
            V_1_0 => match state {
                // Genesis state, displays information about the course and the
                // number of exercises left.
                TestRunnerState::Loaded => {
                    progress.println(DOTCODESCHOOL.clone());

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
                // Displays the name of the current suite
                TestRunnerState::NewSuite(index_suite) => {
                    let suite = &course.suites[index_suite];
                    let suite_name =
                        suite.name.deref().to_uppercase().bold().green();

                    progress.println(format!(
                        "\n{suite_name} {}",
                        if suite.optional { &OPTIONAL } else { "" },
                    ));

                    Self {
                        progress,
                        success,
                        state: TestRunnerState::NewTest(index_suite, 0),
                        course,
                    }
                }
                // Runs the current test. This state is responsible for exiting
                // into a Failed state in case a mandatory test
                // does not pass.
                TestRunnerState::NewTest(index_suite, index_test) => {
                    let suite = &course.suites[index_suite];
                    let test = &suite.tests[index_test];
                    let test_name = test.name.to_lowercase().bold();

                    progress.println(format!(
                        "\n  🧪 Running test {test_name} {}",
                        if test.optional { &OPTIONAL } else { "" },
                    ));

                    progress.inc(1);

                    // Testing happens HERE
                    match test.run() {
                        TestResult::Pass(stdout) => {
                            progress.println(Self::format_output(
                                &stdout,
                                &format!("✅ {}", &test.message_on_success),
                            ));

                            success += 1;
                        }
                        TestResult::Fail(stderr) => {
                            progress.println(
                                Self::format_output(
                                    &stderr,
                                    &format!("❌ {}", &test.message_on_fail),
                                )
                                .red()
                                .dimmed()
                                .to_string(),
                            );

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
                        }
                    };

                    // Moves on to the next text, the next suite, or marks the
                    // tests as Passed
                    match (
                        index_suite + 1 < course.suites.len(),
                        index_test + 1 < suite.tests.len(),
                    ) {
                        (_, true) => Self {
                            progress,
                            success,
                            state: TestRunnerState::NewTest(
                                index_suite,
                                index_test + 1,
                            ),
                            course,
                        },
                        (true, false) => Self {
                            progress,
                            success,
                            state: TestRunnerState::NewSuite(index_suite + 1),
                            course,
                        },
                        (false, false) => Self {
                            progress,
                            success,
                            state: TestRunnerState::Passed,
                            course,
                        },
                    }
                }
                // A mandatory test failed. Displays a custom error message as
                // defined in the `message_on_fail` field of a
                // Test JSON object. This state can also be used for general
                // error logging.
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
                // ALL mandatory tests passed. Displays the success rate across
                // all tests. It is not important how low that
                // rate is, as long as all mandatory tests pass,
                // and simply serves as an indication of progress for the
                // student.
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
                // Exit state, does nothing when called.
                TestRunnerState::Finish => Self {
                    progress,
                    success,
                    state: TestRunnerState::Finish,
                    course,
                },
            },
            // An invalid `tests.json` version has been provided
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
}
