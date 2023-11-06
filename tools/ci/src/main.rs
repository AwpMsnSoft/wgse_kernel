//! ci script copied from [Bevy] engine crates.
//!
//! [Bevy]: https://bevyengine.org/
//!
use anyhow::Result;
use bitflags::bitflags;
use xshell::{cmd, Shell};

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct Check: u32 {
        const DEBUG = 0x10000000;
        const RELEASE = 0x20000000;
        // const EXAMPLE = 0x40000000;
        // const DOC = 0x80000000;

        const FORMAT = 0x00000001;
        const CLIPPY = 0x00000002;
        const CHECK = 0x00000004;
        const TEST = 0x00000008;
    }
}

const CLIPPY_FLAGS: [&str; 6] = [
    "-Wclippy::doc_markdown",
    "-Wclippy::redundant_else",
    "-Wclippy::match_same_arms",
    "-Wclippy::semicolon_if_nothing_returned",
    "-Wclippy::map_flatten",
    "-Dwarnings",
];

fn main() -> Result<()> {
    // When run locally, results may differ from actual CI runs triggered by
    // .github/workflows/ci.yml
    // - Official CI runs latest stable
    // - Local runs use whatever the default Rust is locally

    let arguments = [
        ("format", Check::FORMAT),
        ("clippy", Check::CLIPPY),
        ("lints", Check::FORMAT | Check::CLIPPY),
        ("check", Check::CHECK | Check::DEBUG),
        ("check-release", Check::CHECK | Check::RELEASE),
        // ("check-example", Check::CHECK | Check::EXAMPLE),
        ("test", Check::TEST | Check::DEBUG),
        ("test-release", Check::TEST | Check::RELEASE),
    ];

    let what_to_run = if let Some(arg) = std::env::args().nth(1).as_deref() {
        if let Some((_, check)) = arguments.iter().find(|(str, _)| *str == arg) {
            *check
        } else {
            println!(
                "Invalid argument: {arg:?}.\nEnter one of: {}.",
                arguments[1..]
                    .iter()
                    .map(|(s, _)| s)
                    .fold(arguments[0].0.to_owned(), |c, v| c + ", " + v)
            );
            return Ok(());
        }
    } else {
        Check::all()
    };

    #[allow(unused_assignments)]
    let mut target = "--";

    if what_to_run.contains(Check::RELEASE) {
        target = "--release";
    }

    // if what_to_run.contains(Check::EXAMPLE) {
    //     target = "--example";
    // }

    // if what_to_run.contains(Check::DOC) {
    //     target = "--doc";
    // }

    let sh = Shell::new().unwrap();

    if what_to_run.contains(Check::FORMAT) {
        cmd!(sh, "cargo fmt --all -- --check").run()?;
    }

    if what_to_run.contains(Check::CLIPPY) {
        cmd!(
            sh,
            "cargo clippy --workspace --all-targets --all-features -- {CLIPPY_FLAGS...}"
        )
        .run()?;
    }

    if what_to_run.contains(Check::CHECK) {
        cmd!(sh, "cargo check --workspace {target}").run()?;
    }

    if what_to_run.contains(Check::TEST) {
        cmd!(
            sh,
            "cargo test --workspace --lib --bins --tests --benches {target}"
        )
        .run()?;
    }

    Ok(())
}
