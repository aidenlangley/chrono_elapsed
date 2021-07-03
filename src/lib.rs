use std::convert::TryFrom;

use chrono::{Date, DateTime, Duration, Local, Utc};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Elapsed {
    datetime: DateTime<Local>,
    date: Date<Local>,
    duration: Duration,
    passed: bool,
}

impl Elapsed {
    pub fn new(datetime: DateTime<Local>) -> Self {
        todo!()
        // Self {
        //     datetime,
        //     date,
        //     duration,
        //     passed,
        // }
    }
    pub fn new_from_date(date: Date<Local>) -> Self {
        todo!()
    }
}

impl TryFrom<DateTime<Local>> for Elapsed {
    type Error = &'static str;

    fn try_from(value: DateTime<Local>) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl TryFrom<Date<Local>> for Elapsed {
    type Error = &'static str;

    fn try_from(value: Date<Local>) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl TryFrom<DateTime<Utc>> for Elapsed {
    type Error = &'static str;

    fn try_from(value: DateTime<Utc>) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl TryFrom<Date<Utc>> for Elapsed {
    type Error = &'static str;

    fn try_from(value: Date<Utc>) -> Result<Self, Self::Error> {
        todo!()
    }
}

/**

/**
    Compute the model's `due_in` string and `due_in_unix` timestamp from `due` `date` or `datetime`.
    */
    pub fn compute_due_in(&mut self) -> &Model {
        /* Only applies if the task can be due. */
        if let Some(due) = self.due.as_ref() {
            /* Parse dates and get a `Duration` between now and then. */
            let dt;
            if let Some(datetime) = due.datetime.as_ref() {
                dt = datetime
                    .parse::<chrono::DateTime<chrono::Utc>>()
                    .expect("failed to parse UTC `datetime`")
                    .with_timezone(&chrono::Local);
            } else {
                let parts: Vec<&str> = due.date.split('-').collect();
                let date = chrono::TimeZone::ymd(
                    &chrono::Local,
                    parts[0].parse::<i32>().expect("failed to parse year"),
                    parts[1].parse::<u32>().expect("failed to parse month"),
                    parts[2].parse::<u32>().expect("failed to parse day"),
                );

                /*
                Since we only have `date` without a time, we'll check if it's todays' date and
                return "today" if it is.
                */
                let diff = date.signed_duration_since(chrono::Local::today());
                if diff.num_days() == 0 {
                    self.due_in = Some(String::from("today"));
                    self.due_in_unix = Some(diff.num_seconds());
                    return self;
                }

                dt = date.and_hms(0, 0, 0);
            }

            let now = chrono::Local::now();
            let diff = dt.signed_duration_since(now);

            /* Make hooman readable. */
            fn str_from_diff(diff: chrono::Duration) -> String {
                /*
                All absolute values, we can assume values are below zero later on when we check
                in_future`, whilst we're building the str that represents time elapsed, we aren't
                concerned with past or future.

                `chrono` returns whole weeks, days, etc. so no rounding is present.
                */
                let wk = diff.num_weeks().abs();
                let day = diff.num_days().abs();
                let hr = diff.num_hours().abs();
                let min = diff.num_minutes().abs();
                let sec = diff.num_seconds().abs();

                /*
                We want to return strings like these:
                3y 11m, 1m 2w, 3w, 4d, 1d
                in 4(up to 23)hr, in 1hr 30m
                in 30m, in 02:30s
                */
                if wk > 0 {
                    if wk > 0 && wk < 4 {
                        /* In n weeks, simples. */
                        format!("{}w", wk)
                    } else
                    /* Months: */
                    {
                        /* Round down for months, easy for us to add remaining weeks. */
                        let mon = math::round::floor((wk / 4) as f64, 0) as i64;
                        /*
                        Get remaining weeks, e.g.:
                        6w [1m (+2w, rounded off)] - (1m * 4w) = 2w
                        */
                        let wk_remaining = wk - mon * 4;

                        if mon < 12
                        /* Less than a year: */
                        {
                            format!("{}m, {}w", mon, wk_remaining)
                        } else
                        /* Potentially multiple years */
                        {
                            let yr = math::round::floor((mon / 12) as f64, 0) as i64;
                            let mon_remaining = mon - yr * 12;

                            format!("{}y {}m", yr, mon_remaining)
                        }
                    }
                } else if day > 0
                /* and weeks are 0. */
                {
                    format!("{}d", day)
                } else if hr >= 4
                /* and days are 0. */
                {
                    format!("{}hr", hr)
                } else if min >= 60
                /* and in less than 4 hours. */
                {
                    let min_remaining = min - hr * 60;
                    format!("{}hr {}m", hr, min_remaining)
                } else if min >= 5
                /* and in less than an hour. */
                {
                    let yellow = console::Style::new().yellow();
                    yellow.apply_to(format!("{}m", min)).to_string()
                } else if sec > 0
                /* and less 5 minutes away. */
                {
                    let sec_remaining = sec - min * 60;
                    /* Pads left with 0s. */
                    let red = console::Style::new().red();
                    red.apply_to(format!("{:0>1}:{:0>1}s", min, sec_remaining))
                        .to_string()
                } else {
                    String::from("0s")
                }
            }

            /* Final touches. */
            let val = if diff.gt(&chrono::Duration::zero()) {
                Some(format!("in {}", str_from_diff(diff)))
            } else {
                Some(format!("{} ago", str_from_diff(diff)))
            };

            /* Set. */
            self.due_in = val;
            self.due_in_unix = Some(diff.num_seconds());
        } else {
            self.due_in = None;
            self.due_in_unix = None;
        }

        /* Return. */
        self
    }

*/

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
