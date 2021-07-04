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
