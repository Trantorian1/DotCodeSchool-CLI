use chrono::Local;
use env_logger::Builder;
use log::LevelFilter;
use runner::{TestRunner, TestRunnerState};
use std::{io::Write, os::unix::thread, time::Duration};

mod parsing;
mod runner;

fn main() {
    Builder::from_default_env()
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [{}] - {}",
                Local::now().format("%Y-%m-%dT%H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .init();

    let mut test_runner = TestRunner::new("./tests.json");
    while test_runner.state != TestRunnerState::Finish {
        test_runner = test_runner.run();
        // std::thread::sleep(Duration::from_millis(1000));
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn foo() {
        assert_eq!(0, 0);
    }

    #[test]
    fn bazz() {
        assert_eq!(0, 1);
    }
}
