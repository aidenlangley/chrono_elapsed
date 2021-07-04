use std::{collections::HashMap, convert::TryFrom, fmt::Display};

use chrono::{Date, DateTime, Duration, Local, Utc};
use math::round::floor;

/**
Provides a context-aware `DateTime` object; a given `DateTime` is made aware in the context of the
current `DateTime` (or in the context of a given `DateTime`, see `new_with_context`.)

Allows you to easily obtain information such as `datetime is due in 42 minutes and 30 seconds`,
`datetime is overdue by 3 days and 2 hours`, or when given context, `time between x and y is 42
years and 9 months`.

Aliased as `DueDateTime` out of the box in case that makes more sense in your context.
*/
#[derive(Debug, Clone)]
pub struct Elapsed {
    /**
    The `DateTime` that gives this meaningful context, will default to `now`, but can be modified to
    get elapsed time between dates.
    */
    datetime_context: DateTime<Local>,
    datetime: DateTime<Local>,
    date: Date<Local>,
    /**
    Also known as the `diff`, difference in time between a given `DateTime` and the `DateTime` used
    for context.
    */
    pub duration: Duration,
    /**
    If the date has already `passed`, or `elapsed`, it's no longer `due` so we can skip some
    processing, and format a more meaningful message on display.
    */
    pub passed: bool,
    /**
    A cache of `diff` values.
    Key being the sec/min/hour/day, etc. identifier.
    Value being a tuple of the `diff` value as a string in pos 0, and numeric value in pos 1.
    */
    pub cache: HashMap<TimeFrame, (String, i64)>, // format: &'static str,
}

/* Aliasing `Elapsed` because these names might make more sense, depending on use-case. */
/** Alias of `Elapsed`. */
pub type DueDateTime = Elapsed;
/** Alias of `Elapsed`. */
pub type TimeBetween = Elapsed;

impl Elapsed {
    /** Construct a new object. */
    pub fn new(datetime: DateTime<Local>) -> Self {
        let datetime_context = Local::now();
        let mut obj = Self {
            datetime_context,
            datetime,
            date: datetime.date(),
            duration: datetime.signed_duration_since(datetime_context),
            passed: datetime.le(&datetime_context),
            cache: HashMap::new(),
        };
        obj.process(false);
        obj
    }

    /** Construct a new object from a `Date` rather than `DateTime`. */
    pub fn new_from_date(date: Date<Local>) -> Self {
        let datetime = date.and_hms(0, 0, 0);
        let datetime_context = Local::now();
        let mut obj = Self {
            datetime_context,
            datetime,
            date,
            duration: datetime.signed_duration_since(datetime_context),
            passed: datetime.le(&datetime_context),
            cache: HashMap::new(),
        };
        obj.process(false);
        obj
    }

    /** Construct a new object and then add `Local` timezone. */
    pub fn new_then_localize(datetime: DateTime<Utc>) -> Self {
        let datetime_context = Local::now();
        let datetime = datetime.with_timezone(&Local);
        let mut obj = Self {
            datetime_context,
            datetime,
            date: datetime.date(),
            duration: datetime.signed_duration_since(datetime_context),
            passed: datetime.le(&datetime_context),
            cache: HashMap::new(),
        };
        obj.process(false);
        obj
    }

    /** Construct a new object from a `Date` then add `Local` timezone. */
    pub fn new_from_date_then_localize(date: Date<Utc>) -> Self {
        let datetime = date.and_hms(0, 0, 0).with_timezone(&Local);
        let datetime_context = Local::now();
        let mut obj = Self {
            datetime_context,
            datetime,
            date: datetime.date(),
            duration: datetime.signed_duration_since(datetime_context),
            passed: datetime.le(&datetime_context),
            cache: HashMap::new(),
        };
        obj.process(false);
        obj
    }

    /** Construct a new object with a custom `context`, rather than the default `now`. */
    pub fn new_with_context(datetime: DateTime<Local>, context: DateTime<Local>) -> Self {
        let mut obj = Self {
            datetime_context: context,
            datetime,
            date: datetime.date(),
            duration: datetime.signed_duration_since(context),
            passed: datetime.le(&context),
            cache: HashMap::new(),
        };
        obj.process(false);
        obj
    }

    /** Construct a new object from a `Date` with a custom `context`. */
    pub fn new_from_date_with_context(date: Date<Local>, context: Date<Local>) -> Self {
        let datetime = date.and_hms(0, 0, 0);
        let datetime_context = context.and_hms(0, 0, 0);
        let mut obj = Self {
            datetime_context,
            datetime,
            date,
            duration: datetime.signed_duration_since(datetime_context),
            passed: datetime.le(&datetime_context),
            cache: HashMap::new(),
        };
        obj.process(false);
        obj
    }

    /** Set the `Elapsed`'s datetime_context. Will clear cached `diff` values. */
    pub fn set_datetime_context(&mut self, datetime_context: DateTime<Local>) -> &mut Self {
        self.datetime_context = datetime_context;
        self.duration = self.datetime.signed_duration_since(datetime_context);
        self.passed = self.datetime.le(&self.datetime_context);
        self.process(true);
        self
    }

    /** Set the `Elapsed`'s datetime. Will clear cached `diff` values. */
    pub fn set_datetime(&mut self, datetime: DateTime<Local>) -> &mut Self {
        self.datetime = datetime;
        self.date = datetime.date();
        self.duration = datetime.signed_duration_since(self.datetime_context);
        self.passed = datetime.le(&self.datetime_context);
        self.process(true);
        self
    }

    /** Set the `Elapsed`'s date. Will clear cached `diff` values. */
    pub fn set_date(&mut self, date: Date<Local>) {
        self.date = date;
        self.datetime = date.and_hms(0, 0, 0);
        self.duration = self.datetime.signed_duration_since(self.datetime_context);
        self.passed = self.datetime.le(&self.datetime_context);
        self.process(true);
    }

    /**
    Default behaviour currently. Discards "irrelevant" time frames, for example if date is due in
    more than a year, we'll only store `1y 6m` as opposed to `1y 6m 2w 4d`.
    */
    pub fn process(&mut self, clear_cache: bool) {
        /*
        All absolute values, we can assume values are below zero later on when we check `passed`,
        whilst we're building the str that represents time elapsed, we aren't concerned with past or
        future.

        `chrono` returns whole weeks, days, etc. so no rounding is present.
        */
        let diff = self.duration;
        let weeks = diff.num_weeks().abs();
        let days = diff.num_days().abs();
        let hours = diff.num_hours().abs();
        let minutes = diff.num_minutes().abs();
        let seconds = diff.num_seconds().abs();

        if clear_cache {
            self.cache = HashMap::new();
        }

        if weeks > 0 {
            if weeks > 0 && weeks < 4 {
                /* In n weeks, simples. */
                self.cache_insert(TimeFrame::Week, weeks);
            } else
            /* Months: */
            {
                /* Round down for months, easy for us to add remaining weeks. */
                let months = floor((weeks / 4) as f64, 0) as i64;
                /*
                Get remaining weeks, e.g.:
                6w [1m (+2w, rounded off)] - (1m * 4w) = 2w
                */
                let weeks_remaining = weeks - months * 4;
                if months < 12
                /* Less than a year: */
                {
                    self.cache_insert(TimeFrame::Month, months);
                    self.cache_insert(TimeFrame::Week, weeks_remaining);
                } else
                /* Potentially multiple years */
                {
                    let years = floor((months / 12) as f64, 0) as i64;
                    let months_remaining = months - years * 12;
                    self.cache_insert(TimeFrame::Year, years);
                    self.cache_insert(TimeFrame::Month, months_remaining);
                }
            }
        } else if days > 0
        /* and weeks are 0. */
        {
            self.cache_insert(TimeFrame::Day, days);
        } else if hours >= 4
        /* and days are 0. */
        {
            self.cache_insert(TimeFrame::Hour, hours);
        } else if minutes >= 60
        /* and in less than 4 hours. */
        {
            self.cache_insert(TimeFrame::Minute, minutes - hours * 60);
        } else if minutes >= 5
        /* and in less than an hour. */
        {
            self.cache_insert(TimeFrame::Minute, minutes);
        } else if seconds > 0
        /* and less 5 minutes away. */
        {
            /* Pads left with 0s: format!("{:0>1}:{:0>1}s", min, sec_remaining) */
            self.cache_insert(TimeFrame::Minute, minutes);
            self.cache_insert(TimeFrame::Second, seconds - minutes * 60);
        }
    }

    /** Helper function to insert a value for a `TimeFrame` into the cache. */
    pub fn cache_insert(&mut self, k: TimeFrame, v: i64) {
        let tup = (format!("{}{}", v, k.abbrev()), v);
        let val = self.cache.entry(k).or_insert(tup.clone());
        *val = tup;
    }

    fn years(&mut self) {
        todo!()
    }

    fn months(&mut self) {
        todo!()
    }

    fn weeks(&mut self) {
        todo!()
    }

    fn days(&mut self) {
        todo!()
    }

    fn hours(&mut self) {
        todo!()
    }

    fn minutes(&mut self) {
        todo!()
    }

    fn seconds(&mut self) {
        todo!()
    }
}

impl Display for Elapsed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut vec: Vec<&str> = Vec::new();
        if let Some(years) = self.cache.get(&TimeFrame::Year) {
            vec.push(&years.0);
        }
        if let Some(months) = self.cache.get(&TimeFrame::Month) {
            vec.push(&months.0);
        }
        if let Some(weeks) = self.cache.get(&TimeFrame::Week) {
            vec.push(&weeks.0);
        }
        if let Some(days) = self.cache.get(&TimeFrame::Day) {
            vec.push(&days.0);
        }
        if let Some(hours) = self.cache.get(&TimeFrame::Hour) {
            vec.push(&hours.0);
        }
        if let Some(minutes) = self.cache.get(&TimeFrame::Minute) {
            vec.push(&minutes.0);
        }
        if let Some(seconds) = self.cache.get(&TimeFrame::Second) {
            vec.push(&seconds.0);
        }

        if self.passed {
            write!(f, "{} ago", vec.join(" "))
        } else {
            write!(f, "in {}", vec.join(" "))
        }
    }
}

impl From<DateTime<Local>> for Elapsed {
    /** Construct _from_ localised `DateTime`. */
    fn from(datetime: DateTime<Local>) -> Self {
        Self::new(datetime)
    }
}

impl From<Date<Local>> for Elapsed {
    /** Construct _from_ localised `Date`. */
    fn from(date: Date<Local>) -> Self {
        Self::new_from_date(date)
    }
}

impl From<DateTime<Utc>> for Elapsed {
    /** Construct _from_ UTC `DateTime`. */
    fn from(datetime: DateTime<Utc>) -> Self {
        Self::new_then_localize(datetime)
    }
}

impl From<Date<Utc>> for Elapsed {
    /** Construct _from_ UTC `Date`. */
    fn from(date: Date<Utc>) -> Self {
        Self::new_from_date_then_localize(date)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TimeFrame {
    /*
    Tempted to leave millisecond out because by virtue this crate isn't dealing with micro and nano
    seconds, but milliseconds are useful in the Unix world. A millisecond to us wouldn't ever be
    more than 60 however.
    */
    MilliSecond,
    Second,
    Minute,
    Hour,
    Day,
    Week,
    Month,
    Year,
    // Decade ...
}

impl From<TimeFrame> for String {
    /** Return `String` from `TimeFrame`. */
    fn from(tf: TimeFrame) -> Self {
        match tf {
            TimeFrame::MilliSecond => String::from("millisecond(s)"),
            TimeFrame::Second => String::from("second(s)"),
            TimeFrame::Minute => String::from("minute(s)"),
            TimeFrame::Hour => String::from("hour(s)"),
            TimeFrame::Day => String::from("day(s)"),
            TimeFrame::Week => String::from("week(s)"),
            TimeFrame::Month => String::from("month(s)"),
            TimeFrame::Year => String::from("year(s)"),
        }
    }
}

impl TryFrom<&str> for TimeFrame {
    type Error = &'static str;
    /** Attempt to parse `str` to `TimeFrame`. */
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().trim() {
            "millisecond" | "ms" => Ok(Self::MilliSecond),
            "second" | "sec" | "s" => Ok(Self::Second),
            "minute" | "min" => Ok(Self::Minute),
            "hour" | "hr" | "h" => Ok(Self::Hour),
            "day" | "d" => Ok(Self::Day),
            "week" | "wk" | "w" => Ok(Self::Week),
            "month" | "mon" => Ok(Self::Month),
            "year" | "yr" | "y" => Ok(Self::Year),
            _ => Err("Invalid or ambiguous string for `elapsed::TimeFrame`"),
        }
    }
}

impl From<TimeFrame> for char {
    /** Return `char` from `TimeFrame`. */
    fn from(tf: TimeFrame) -> Self {
        match tf {
            TimeFrame::MilliSecond => 'm',
            TimeFrame::Second => 's',
            TimeFrame::Minute => 'm',
            TimeFrame::Hour => 'h',
            TimeFrame::Day => 'd',
            TimeFrame::Week => 'w',
            TimeFrame::Month => 'm',
            TimeFrame::Year => 'y',
        }
    }
}

impl TryFrom<char> for TimeFrame {
    type Error = &'static str;
    /** Attempt to parse `char` to `TimeFrame`. clashes will fail (m for ms, min, month). */
    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value.to_ascii_lowercase() {
            's' => Ok(Self::Second),
            'h' => Ok(Self::Hour),
            'd' => Ok(Self::Day),
            'w' => Ok(Self::Week),
            'y' => Ok(Self::Year),
            _ => Err("Invalid or ambiguous char for `elapsed::TimeFrame`"),
        }
    }
}

pub trait Abbreviate {
    fn abbrev(&self) -> &'static str;
    fn abbrev_short(&self) -> &'static str;
}

impl Abbreviate for TimeFrame {
    /** Abbreviate `TimeFrame` to reasonably short string. */
    fn abbrev(&self) -> &'static str {
        match self {
            TimeFrame::MilliSecond => "ms",
            TimeFrame::Second => "sec",
            TimeFrame::Minute => "min",
            TimeFrame::Hour => "hr",
            TimeFrame::Day => "d",
            TimeFrame::Week => "w",
            TimeFrame::Month => "m",
            TimeFrame::Year => "y",
        }
    }

    /**
    Abbreviate `TimeFrame` to a still sensibly short string, mostly just a char except when there
    are clashes (ms, min, month).
    */
    fn abbrev_short(&self) -> &'static str {
        match self {
            TimeFrame::MilliSecond => "ms",
            TimeFrame::Second => "s",
            TimeFrame::Minute => "min",
            TimeFrame::Hour => "h",
            TimeFrame::Day => "d",
            TimeFrame::Week => "w",
            TimeFrame::Month => "m",
            TimeFrame::Year => "y",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn print_elapsed_since_birth() {
        let dt_str = "1993-10-30T04:20:00Z";
        let past_dt = dt_str
            .parse::<DateTime<Local>>()
            .expect("failed to parse str as `DateTime<Local>`");
        let elapsed = Elapsed::new(past_dt);
        println!("{}", elapsed)
    }

    #[test]
    fn print_elapsed_since_recent() {
        let now = Local::now();
        let recent_dt = now - Duration::minutes(20);
        let elapsed = Elapsed::new(recent_dt);
        println!("{}", elapsed)
    }
}
