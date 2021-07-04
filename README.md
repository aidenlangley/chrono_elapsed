# chrono_elapsed

An extension of [chrono](https://crates.io/crates/chrono) to assist in reporting
on due dates. I wrote this because I'm  building a [CLI](https://github.com/aidenlangley/clogi)
in Rust and I chose to bring [Todoist](https://todoist.com) to the terminal
since I primarily use it for my todo's, and wanted to integrate it more closely
with my dev workflow.

Personally, I'm only using this for dates that are in the near future, or have
very recently passed, so it doesn't currently accurately handle oddities in the
calendar, such as February. A lot of assumptions are being made, so fair warning
if you're interested in a crate for getting accurate data regarding time elapsed
between two dates, __this is not for you__ (yet... I hope.)

## Example

```rust
let dt_str = "1993-10-30T04:20:00Z";
let past_dt = dt_str
    .parse::<DateTime<Local>>()
    .expect("failed to parse str as `DateTime<Local>`");
let elapsed = Elapsed::new(past_dt);
println!("{}", elapsed);
```

Would print: `30y 1m`. Wildly inaccurate, but you get the idea. Becomes more
accurate when the date is closer to now. A closer datetime that had just passed
would print something like `4min 46sec`, e.g.

```rust
let now = Local::now();
let recent_dt = now - Duration::minutes(20);
let elapsed = Elapsed::new(recent_dt);
println!("{}", elapsed)
```
