use chrono::{DateTime, Duration, Local, Utc};
use math::round::floor;
use std::{borrow::Cow, convert::TryFrom, fmt::Display, u64};

/// Provides a context-aware `DateTime` object; a given `DateTime` is made
/// aware in the context of the current `DateTime` (or in the context of a
/// given `DateTime`, see `new_with_context`.)
///
/// Allows you to easily obtain information such as `datetime is due in
/// 42 minutes and 30 seconds`, `datetime is overdue by 3 days and 2 hours`, or
/// when given context, `time between x and y is 42 years and 9 months`.
///
/// Aliased as `DueDateTime` out of the box in case that makes more sense in
/// your context.
#[derive(Debug, Clone)]
pub struct Elapsed {
    /// The `DateTime` that gives this meaningful context, will default to `now`,
    /// but can be modified to get elapsed time between dates.
    datetime_context: DateTime<Local>,
    datetime: DateTime<Local>,

    /// Also known as the `diff`, difference in time between a given `DateTime`
    /// and the `DateTime` used for context.
    pub duration: Duration,

    /// If the date has already `passed`, or `elapsed`, it's no longer `due` so
    /// we can skip some processing, and format a more meaningful message on
    /// display.
    pub passed: bool,

    /// No need for a HashMap. We want an ordered array, and we will use fixed
    /// positioning so we can retrieve the data in the same order as the enum
    /// `TimeFrame`. Contains `Option<T>` because we'd like to allow `None`
    /// when user is not concerned with a particular time frame.
    ///
    /// We store a tuple for flexibility. Usually, we're just going to pull out
    /// the string, but there might be times when we want the raw `u64`.
    pub cache: Cache,
    /*
    TODO:
    Customising display format can be done here.
    */
    // format: &'static str,
    /*
    TODO:
    epoch could eventually be useful for running a timer that can be started via CLI.
    A tick of the clock would then be a trigger to calculate if a minute had elapsed.
    */
    // epoch: u64,
}

/// Alias of `Elapsed`.
pub type DueDateTime = Elapsed;

/// Alias of `Elapsed`.
pub type TimeBetween = Elapsed;

/// Private `TimeFrameTuple` type to avoid duplicate code.
type TimeFrameTuple = (Cow<'static, str>, u64);

/// Private `Cache` type to avoid duplicate code. Note: remember to change size
/// here if number of enum variants changes.
type Cache = [Option<TimeFrameTuple>; 8];

impl Elapsed {
    /// Construct a new object then immediately process it.
    pub fn new(datetime: DateTime<Local>) -> Self {
        let mut obj = Self::custom(datetime);
        obj.process();
        obj
    }

    /// Construct a new object and then add `Local` timezone then immediately
    /// process it.
    pub fn new_then_localize(datetime: DateTime<Utc>) -> Self {
        let mut obj = Self::custom_then_localize(datetime);
        obj.process();
        obj
    }

    /// Construct a new object with a custom `context`, rather than the default
    /// `now` then immediately process it.
    pub fn new_with_context(datetime: DateTime<Local>, context: DateTime<Local>) -> Self {
        let mut obj = Self::custom_with_context(datetime, context);
        obj.process();
        obj
    }

    /// Construct a new object without processing. You must select the values to
    /// calculate via `years` or a sequence `years_and`, etc.
    pub fn custom(datetime: DateTime<Local>) -> Self {
        let datetime_context = Local::now();
        Self {
            datetime_context,
            datetime,
            duration: datetime.signed_duration_since(datetime_context),
            passed: datetime.le(&datetime_context),
            cache: Cache::default(),
        }
    }

    /// Construct a new object and then add `Local` timezone without processing.
    /// You must select the values to calculate via `years` or a sequence
    /// `years_and`, etc.
    pub fn custom_then_localize(datetime: DateTime<Utc>) -> Self {
        let datetime_context = Local::now();
        let datetime = datetime.with_timezone(&Local);
        Self {
            datetime_context,
            datetime,
            duration: datetime.signed_duration_since(datetime_context),
            passed: datetime.le(&datetime_context),
            cache: Cache::default(),
        }
    }

    /// Construct a new object with a custom `context`, rather than the default
    /// `now` without processing. You must select the values to calculate via
    /// `years` or a sequence `years_and`, etc.
    pub fn custom_with_context(datetime: DateTime<Local>, context: DateTime<Local>) -> Self {
        Self {
            datetime_context: context,
            datetime,
            duration: datetime.signed_duration_since(context),
            passed: datetime.le(&context),
            cache: Cache::default(),
        }
    }

    /// Set the `Elapsed`'s datetime_context. Will clear cached `diff` values.
    pub fn set_datetime_context(&mut self, datetime_context: DateTime<Local>) -> &mut Self {
        self.datetime_context = datetime_context;
        self.duration = self.datetime.signed_duration_since(datetime_context);
        self.passed = self.datetime.le(&self.datetime_context);
        self.clear_cache();
        self.process();
        self
    }

    /// Set the `Elapsed`'s datetime. Will clear cached `diff` values.
    pub fn set_datetime(&mut self, datetime: DateTime<Local>) -> &mut Self {
        self.datetime = datetime;
        self.duration = datetime.signed_duration_since(self.datetime_context);
        self.passed = datetime.le(&self.datetime_context);
        self.clear_cache();
        self.process();
        self
    }

    /// Populate `cache` with contextually aware `TimeFrame`s. Discards
    /// "irrelevant" time frames, for example if date is due in more than a year,
    /// we'll only store `1y 6m` as opposed to `1y 6m 2w 4d`.
    pub fn process(&mut self) {
        // All absolute values, we can assume values are below zero later on
        // when we check `passed`, whilst we're building the str that represents
        // time elapsed, we aren't concerned with past or future.
        //
        // `chrono` returns whole weeks, days, etc. so no rounding is present.
        let diff = self.duration;
        let weeks = diff.num_weeks().unsigned_abs();
        let days = diff.num_days().unsigned_abs();
        let hours = diff.num_hours().unsigned_abs();
        let minutes = diff.num_minutes().unsigned_abs();
        let seconds = diff.num_seconds().unsigned_abs();
        let _milliseconds = diff.num_milliseconds().unsigned_abs();

        if weeks > 0 {
            if weeks > 0 && weeks < 4 {
                // In n weeks, simples.
                self.cache_insert(TimeFrame::Week, weeks);
            } else {
                // Months:
                // Round down for months, easy for us to add remaining weeks.
                let months = floor((weeks / 4) as f64, 0) as u64;

                // Get remaining weeks, e.g.:
                // 6w [1m (+2w, rounded off)] - (1m * 4w) = 2w
                let weeks_remaining = weeks - months * 4;

                if months < 12 {
                    // Less than a year:
                    self.cache_insert(TimeFrame::Month, months);
                    self.cache_insert(TimeFrame::Week, weeks_remaining);
                } else {
                    // Potentially multiple years
                    let years = floor((months / 12) as f64, 0) as u64;
                    let months_remaining = months - years * 12;
                    self.cache_insert(TimeFrame::Year, years);
                    self.cache_insert(TimeFrame::Month, months_remaining);
                }
            }
        } else if days > 0 {
            // and weeks are 0.
            self.cache_insert(TimeFrame::Day, days);
        } else if hours >= 4 {
            // and days are 0.
            self.cache_insert(TimeFrame::Hour, hours);
        } else if minutes >= 60 {
            // and in less than 4 hours.
            self.cache_insert(TimeFrame::Minute, minutes - hours * 60);
        } else if minutes >= 5 {
            // and in less than an hour.
            self.cache_insert(TimeFrame::Minute, minutes);
        } else if seconds > 0 {
            // and less 5 minutes away.

            // Pads left with 0s: format!("{:0>1}:{:0>1}s", min, sec_remaining)
            self.cache_insert(TimeFrame::Minute, minutes);
            self.cache_insert(TimeFrame::Second, seconds - minutes * 60);
        }
    }

    /// Helper fn to insert a value for a `TimeFrame` into the cache.
    pub fn cache_insert(&mut self, k: TimeFrame, v: u64) {
        self.cache[k as usize] = Some(Self::as_tuple(k, v));
    }

    /// Helper fn to keep the user in check before throwing wack values in the
    /// `cache`.
    fn protected_insert(&mut self, k: TimeFrame, v: u64) {
        for i in 0..k as usize {
            if self.cache[i].is_some() {
                panic!(
                    "Please, let's try and be civil. Make your calls from largest `TimeFrame` to smallest."
                )
            }
        }
        self.cache_insert(k, v);
    }

    /// Helper fn to clear `HashMap`, bit unnecessary.
    pub fn clear_cache(&mut self) {
        if !self.cache.is_empty() {
            self.cache = Cache::default();
        }
    }

    /// Get number of years.
    pub fn num_years(&self) -> u64 {
        floor((self.duration.num_weeks() / 52) as f64, 0) as u64
    }

    /// Get years between `DateTime` and `DateTime` given for context as
    /// `elapsed` style tuple.
    pub fn years(&mut self) -> TimeFrameTuple {
        Self::as_tuple(TimeFrame::Year, self.num_years())
    }

    /// Get years between `DateTime` and `DateTime` given for context. Can be
    /// chained to string together multiple values of your choosing; `cache`
    /// must be clear before doing this.
    ///
    /// ```rust
    /// let dt = Local::now();
    /// let mut elapsed = Elapsed::custom(dt);
    /// println!("{}", elapsed.years_and().months_and().weeks());
    /// elapsed.clear_cache();
    /// // This one is silly.
    /// println!("{}", elapsed.years_and().seconds());
    /// ```
    ///
    /// Results in `1y 6m 2w` the first time, or something silly the second time.
    ///
    /// Will panic if you do something extra silly like `elapsed.seconds_and().years()`
    /// (even though it doesn't seem _that_ silly.) I have to enforce _some_ rules.
    pub fn years_and(&mut self) -> &mut Self {
        self.protected_insert(TimeFrame::Year, self.num_years());
        self
    }

    /// Get number of months.
    pub fn num_months(&self) -> u64 {
        floor((self.duration.num_weeks() / 4) as f64, 0) as u64
    }

    /// Get months between `DateTime` and `DateTime` given for context as
    /// `elapsed` style tuple.
    pub fn months(&mut self) -> TimeFrameTuple {
        let mut months = self.num_months();
        if let Some(years) = &self.cache[TimeFrame::Year as usize] {
            months -= years.1;
        }
        Self::as_tuple(TimeFrame::Month, months)
    }

    pub fn months_and(&mut self) -> &mut Self {
        let months = self.months().1 - (self.num_years() * 12);
        self.protected_insert(TimeFrame::Month, months);
        self
    }

    /// Get weeks between `DateTime` and `DateTime` given for context as
    /// `elapsed` style tuple.
    ///
    /// Chrono provides a method to get numeric value alone, which is exposed
    /// by `Elapsed` struct `duration` field.
    pub fn weeks(&mut self) -> TimeFrameTuple {
        Self::as_tuple(TimeFrame::Week, self.duration.num_weeks() as u64)
    }

    /// Get days between `DateTime` and `DateTime` given for context as
    /// `elapsed` style tuple.
    ///
    /// Chrono provides a method to get numeric value alone, which is exposed
    /// by `Elapsed` struct `duration` field.
    pub fn days(&mut self) -> TimeFrameTuple {
        Self::as_tuple(TimeFrame::Day, self.duration.num_days() as u64)
    }

    /// Get hours between `DateTime` and `DateTime` given for context as
    /// `elapsed` style tuple.
    ///
    /// Chrono provides a method to get numeric value alone, which is exposed
    /// by `Elapsed` struct `duration` field.
    pub fn hours(&mut self) -> TimeFrameTuple {
        Self::as_tuple(TimeFrame::Hour, self.duration.num_hours() as u64)
    }

    /// Get minutes between `DateTime` and `DateTime` given for context as
    /// `elapsed` style tuple.
    ///
    /// Chrono provides a method to get numeric value alone, which is exposed
    /// by `Elapsed` struct `duration` field.
    pub fn minutes(&mut self) -> TimeFrameTuple {
        Self::as_tuple(TimeFrame::Minute, self.duration.num_minutes() as u64)
    }

    /// Get seconds between `DateTime` and `DateTime` given for context as
    /// `elapsed` style tuple.
    ///
    /// Chrono provides a method to get numeric value alone, which is exposed
    /// by `Elapsed` struct `duration` field.
    pub fn seconds(&mut self) -> TimeFrameTuple {
        const _SEC_IN_MIN: u64 = 60;
        const _SEC_IN_HOUR: u64 = _SEC_IN_MIN * 60;
        const _SEC_IN_DAY: u64 = _SEC_IN_HOUR * 24;
        const _SEC_IN_WEEK: u64 = _SEC_IN_DAY * 7;
        Self::as_tuple(TimeFrame::Second, self.duration.num_seconds() as u64)
    }

    /// Helper fn to get an elapsed style tuple.
    fn as_tuple(tf: TimeFrame, val: u64) -> TimeFrameTuple {
        (format!("{}{}", val, tf.abbrev()).into(), val)
    }

    /// This fn is intended to be used similarly to chaining, like so:
    ///
    /// ```rust
    /// let date = self.seconds_and().through_til(&TimeFrame::Months);
    /// println!("{}", date);
    /// ```
    ///
    /// Resulting in seconds, minutes, hours, days, weeks and months being set
    /// in `cache`, and then subsequently printed as
    /// `(in) 3y 2w 4d 12hr 32min 42sec (ago)`.
    pub fn through_til(&mut self, _tf: &TimeFrame) -> &mut Self {
        todo!()
    }

    /// Create a clone of our `cache` containing the values at time of collection.
    pub fn collect(&self) -> Cache {
        self.cache.clone()
    }
}

impl Display for Elapsed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut vec: Vec<&str> = Vec::new();
        if let Some(years) = &self.cache[TimeFrame::Year as usize] {
            vec.push(&years.0);
        }
        if let Some(months) = &self.cache[TimeFrame::Month as usize] {
            vec.push(&months.0);
        }
        if let Some(weeks) = &self.cache[TimeFrame::Week as usize] {
            vec.push(&weeks.0);
        }
        if let Some(days) = &self.cache[TimeFrame::Day as usize] {
            vec.push(&days.0);
        }
        if let Some(hours) = &self.cache[TimeFrame::Hour as usize] {
            vec.push(&hours.0);
        }
        if let Some(minutes) = &self.cache[TimeFrame::Minute as usize] {
            vec.push(&minutes.0);
        }
        if let Some(seconds) = &self.cache[TimeFrame::Second as usize] {
            vec.push(&seconds.0);
        }
        if let Some(milliseconds) = &self.cache[TimeFrame::MilliSecond as usize] {
            vec.push(&milliseconds.0);
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

impl From<DateTime<Utc>> for Elapsed {
    /** Construct _from_ UTC `DateTime`. */
    fn from(datetime: DateTime<Utc>) -> Self {
        Self::new_then_localize(datetime)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum TimeFrame {
    // Tempted to leave millisecond out because by virtue this crate isn't
    // dealing with micro and nano seconds, but milliseconds are useful in the
    // Unix world. A millisecond to us wouldn't ever be more than 60 however.
    MilliSecond = 0,
    Second = 1,
    Minute = 2,
    Hour = 3,
    Day = 4,
    Week = 5,
    Month = 6,
    Year = 7,
    // Decade ...
}

impl From<TimeFrame> for String {
    /// Return `String` from `TimeFrame`.
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
    /// Attempt to parse `str` to `TimeFrame`.
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
    /// Return `char` from `TimeFrame`.
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
    /// Attempt to parse `char` to `TimeFrame`,
    /// clashes will fail (m for ms, min, month).
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
    /// Abbreviate `TimeFrame` to reasonably short string.
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

    /// Abbreviate `TimeFrame` to a still sensibly short string, mostly just a
    /// char except when there are clashes (ms, min, month).
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
