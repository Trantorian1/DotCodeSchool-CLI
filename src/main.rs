use chrono::Local;
use env_logger::Builder;
use runner::{Runner, RunnerVersion, TestRunnerState};
use std::io::Write;

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

    let mut runner = RunnerVersion::new("./tests.json");
    while runner.state() != TestRunnerState::Finish {
        runner = runner.run();
    }
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    #[test]
    fn foo() {
        std::thread::sleep(Duration::from_millis(500));
        assert_eq!(0, 0);
    }

    #[test]
    fn bazz() {
        std::thread::sleep(Duration::from_millis(500));
        assert_eq!(0, 1);
    }
}
