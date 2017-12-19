extern crate chrono;
extern crate config;
#[macro_use]
extern crate failure;
extern crate hubcaps;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate tokio_core;
extern crate unicorn_hat_hd;

use failure::Error;
use hubcaps::{Credentials, Github};
use hubcaps::issues;
use hubcaps::issues::IssueListOptionsBuilder;
use tokio_core::reactor::Core;
use unicorn_hat_hd::{UnicornHatHd, Rotate};

mod settings;

fn main() {
    let mut uhd = setup_unicorn_hat_hd();

    let settings = match settings::Settings::new() {
        Ok(s) => s,
        Err(e) => panic!("Could not read settings: {}", e),
    };
    let token = settings.github_token;

    let mut core = Core::new().expect("reactor fail");
    let github = Github::new(
        concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")),
        Some(Credentials::Token(token)),
        &core.handle(),
    );

    for repo in settings.repositories {
        let mut open_issues = 0;
        let mut closed_issues = 0;
        let mut assigned_open_issues = 0;

        // Gather information about the open issues.
        for issue in core.run(github.repo(repo.user.clone(), repo.name.clone()).issues().list(&{
            let mut ilo = IssueListOptionsBuilder::new();
            ilo.state(issues::State::Open);
            if let Some(ref l) = repo.labels {
                ilo.labels(l.clone());
            };

            ilo.build()
        })).unwrap() {
            open_issues += 1;

            match issue.assignee {
                Some(_) => {
                    if issue.closed_at == None {
                        assigned_open_issues += 1
                    }
                },
                None => {},
            };
        }

        // Gather information about the closed issues that were updated more
        // recently than repo.since.
        for issue in core.run(github.repo(repo.user.clone(), repo.name.clone()).issues().list(&{
            let mut ilo = IssueListOptionsBuilder::new();
            ilo.state(issues::State::Closed);
            if let Some(ref l) = repo.labels {
                ilo.labels(l.clone());
            };
            if let Some(ref s) = repo.since {
                ilo.since(s.clone());
            }

            ilo.build()
        })).unwrap() {
            closed_issues += 1;
        }

        println!("Summary ({} issues):", open_issues + closed_issues);
        println!("\tOpen: {}", open_issues);
        println!("\tClosed: {}", closed_issues);
        println!("\tAssigned: {}", assigned_open_issues);

        fill_column_ratio(&mut uhd, 0, vec![open_issues, closed_issues], vec![(0, 255, 0), (0, 0, 255)]);
        uhd.display();
    }
}

fn setup_unicorn_hat_hd() -> UnicornHatHd {
    let mut uhd = UnicornHatHd::default();
    uhd.set_rotation(Rotate::Rot180);
    uhd.display().unwrap();

    uhd
}

fn fill_column(mut uhd: &mut UnicornHatHd, col: usize, colors: Vec<(u8, u8, u8)>) -> Result<(), Error> {
    if colors.len() > 16 {
        return Err(format_err!("Number of values ({}) cannot exceed 16.", colors.len()));
    }

    for (i, &(r, g, b)) in colors.iter().enumerate() {
        uhd.set_pixel(col, 15 - i, r, g, b);
    }

    Ok(())
}

fn fill_column_ratio(mut uhd: &mut UnicornHatHd, col: usize, vals: Vec<u32>, colors: Vec<(u8, u8, u8)>) -> Result<(), Error> {
    if vals.len() != colors.len() {
        return Err(format_err!("Number of values ({}) does not match number of colors ({}).", vals.len(), colors.len()));
    }

    let mut leds = vec![];
    let total: u32 = vals.iter().sum();
    for (i, &v) in vals.iter().enumerate() {
        let num_leds = ((16f64 * (f64::from(v) / f64::from(total))).round()) as u64;
        for _ in 0..num_leds {
            leds.push(colors[i]);
        }
    }

    fill_column(&mut uhd, col, leds)
}
