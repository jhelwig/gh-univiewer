extern crate failure;
extern crate hubcaps;
extern crate tokio_core;

#[cfg(feature = "unicorn")]
extern crate unicorn_hat_hd;

use std::env;

use hubcaps::{Credentials, Github};
use hubcaps::issues;
use hubcaps::issues::IssueListOptionsBuilder;
use tokio_core::reactor::Core;

#[cfg(feature = "unicorn")]
use unicorn_hat_hd::{UnicornHatHd, Rotate};

fn main() {
    let mut uhd = setup_unicorn_hat_hd();

    let mut core = Core::new().expect("reactor fail");
    let token = env::var("GH_UNIVIEWER_GITHUB_TOKEN").expect("Missing GH_UNIVIEWER_GITHUB_TOKEN");
    let github = Github::new(
        concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")),
        Some(Credentials::Token(token)),
        &core.handle(),
    );
    let mut open_issues = 0;
    let mut closed_issues = 0;
    let mut community_issues = 0;
    for issue in core.run(github.repo("puppetlabs", "puppet").issues().list(
        &IssueListOptionsBuilder::new()
            .state(issues::State::All)
            .labels(vec!["Community"])
            .build()
        )
    ).unwrap() {
        // println!("{:#?}", issue);
        match issue.closed_at {
            Some(_) => closed_issues += 1,
            None => open_issues += 1,
        };
        let mut labels = vec![];
        for label in issue.labels {
            if label.name == "Community" {
                community_issues += 1;
            }
            labels.push(label.name);
        }
        println!("Labels: {}", labels.join(", "));
    }

    println!("Summary ({} issues):", open_issues + closed_issues);
    println!("\tOpen: {}", open_issues);
    println!("\tClosed: {}", closed_issues);
    println!("\tCommunity: {}", community_issues);
}

#[cfg(feature = "unicorn")]
fn setup_unicorn_hat_hd() -> Option<UnicornHatHd> {
    let mut uhd = UnicornHatHd::default();
    uhd.set_rotation(Rotate::Rot180);
    uhd.display().unwrap();

    Some(uhd)
}

#[cfg(not(feature = "unicorn"))]
struct UnicornHatHd;

#[cfg(not(feature = "unicorn"))]
fn setup_unicorn_hat_hd() -> Option<UnicornHatHd> {
    None
}
