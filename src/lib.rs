use chrono::{Date, DateTime, Duration, Local};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Elapsed {
    datetime: DateTime<Local>,
    date: Date<Local>,
    duration: Duration,
    passed: bool,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
