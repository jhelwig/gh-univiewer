extern crate chrono;
extern crate config;
#[macro_use]
extern crate failure;
extern crate hubcaps;
extern crate rgb;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate tokio_core;
extern crate unicorn_hat_hd;

#[cfg(test)]
#[macro_use]
extern crate spectral;

use failure::Error;
use hubcaps::{Credentials, Github};
use hubcaps::issues;
use hubcaps::issues::IssueListOptionsBuilder;
use rgb::RGB8;
use tokio_core::reactor::Core;
use unicorn_hat_hd::{UnicornHatHd, Rotate};

mod settings;
mod display;

use std::thread;
use std::time::Duration;

use display::MetricType;

fn main() {
    let mut uhd = setup_unicorn_hat_hd();

    let settings = match settings::Settings::new() {
        Ok(s) => s,
        Err(e) => panic!("Could not read settings: {}", e),
    };

    loop {
        update_display(&settings, &mut uhd);
        thread::sleep(Duration::from_secs(600));
    }
}

fn update_display(settings: &settings::Settings, mut uhd: &mut UnicornHatHd) {
    let token = settings.github_token.clone();

    let mut core = Core::new().expect("reactor fail");
    let github = Github::new(
        concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")),
        Some(Credentials::Token(token)),
        &core.handle(),
    );

    let mut metrics = vec![];

    for repo in &settings.repositories {
        let hubcap_repo = github.repo(repo.user.clone(), repo.name.clone());
        let mut open_issues = 0;
        let mut closed_issues = 0;
        let mut merged_issues = 0;
        let mut assigned_open_issues = 0;

        // Gather information about the open issues.
        for issue in core.run(hubcap_repo.issues().list(&{
            let mut ilo = IssueListOptionsBuilder::new();
            ilo.state(issues::State::Open);
            if let Some(ref l) = repo.labels {
                ilo.labels(l.clone());
            };

            ilo.build()
        })).unwrap() {
            open_issues += 1;

            if let Some(_) = issue.assignee {
                if issue.closed_at == None {
                    assigned_open_issues += 1;
                }
            };
        }

        // Gather information about the closed issues that were updated more
        // recently than repo.since.
        for issue in core.run(hubcap_repo.issues().list(&{
            let mut ilo = IssueListOptionsBuilder::new();
            ilo.state(issues::State::Closed);
            if let Some(ref l) = repo.labels {
                ilo.labels(l.clone());
            };
            if let Some(s) = repo.closed_since_date() {
                ilo.since(format!("{}", s.format("%Y-%m-%d")));
            }

            ilo.build()
        })).unwrap() {
            closed_issues += 1;

            let pr = core.run(hubcap_repo.pulls().get(issue.number).get()).unwrap();
            if let Some(_) = pr.merged_at {
                merged_issues += 1;
            }
        }

        println!("Summary ({} issues):", open_issues + closed_issues);
        println!("\tOpen: {}", open_issues);
        println!("\tClosed: {}", closed_issues);
        println!("\tMerged: {}", merged_issues);
        println!("\tAssigned: {}", assigned_open_issues);

        metrics.push(MetricType::ColumnRatio {
            width: 1,
            values: vec![open_issues, closed_issues - merged_issues, merged_issues],
            colors: vec![RGB8::new(0, 255, 0), RGB8::new(0, 0, 255), RGB8::new(191, 119, 246)]
        });

        metrics.push(MetricType::ColumnRatio {
            width: 1,
            values: vec![open_issues - assigned_open_issues, assigned_open_issues],
            colors: vec![RGB8::new(12,255,12), RGB8::new(2,171,46)]
        });
    }

    display_metrics(&mut uhd, metrics);
}

fn setup_unicorn_hat_hd() -> UnicornHatHd {
    let mut uhd = UnicornHatHd::default();
    uhd.set_rotation(Rotate::Rot180);

    uhd
}

fn fill_column(uhd: &mut UnicornHatHd, col: usize, colors: Vec<RGB8>) -> Result<(), Error> {
    if colors.len() > 16 {
        return Err(format_err!("Number of values ({}) cannot exceed 16.", colors.len()));
    }

    for (i, &c) in colors.iter().enumerate() {
        uhd.set_pixel(col, 15 - i, c);
    }

    Ok(())
}

fn vector_of_leds(vals: Vec<u32>) -> Vec<u64> {
    let total: u32 = vals.iter().sum();

    let mut return_vector_float: Vec<f64> = Vec::new();
    let mut return_vector_round: Vec<u64> = Vec::new();

    for &val in &vals {
        let ret_val = if total == 0 {
            0f64
        } else {
            16f64 * (f64::from(val) / f64::from(total))
        };
        return_vector_float.push(ret_val.clone());
        return_vector_round.push(ret_val.round() as u64);
    }

    let round_sum = return_vector_round.iter().sum::<u64>();
    if round_sum > 16 {
        let min = return_vector_float.iter().fold(100.0f64, |acc, &x| {
            if x > 1.5 {
                acc.min(x)
            } else {
                acc
            }
        });
        let mut final_vector: Vec<u64> = Vec::new();
        for val in &return_vector_float {
            if val == &min {
                final_vector.push((val - 1.0).round() as u64);
            } else {
                final_vector.push(val.round() as u64);
            }
        }
        final_vector
    } else {
        return_vector_round
    }
}

fn fill_column_ratio(mut uhd: &mut UnicornHatHd, col: usize, vals: Vec<u32>, colors: Vec<RGB8>) -> Result<(), Error> {
    if vals.len() != colors.len() {
        return Err(format_err!("Number of values ({}) does not match number of colors ({}).", vals.len(), colors.len()));
    }

    let mut leds = vec![];

    let num_leds_vector = vector_of_leds(vals.clone());
    for (i, &num_leds) in num_leds_vector.iter().enumerate() {
        for _ in 0..num_leds {
            leds.push(colors[i].clone());
        }
    }

    fill_column(&mut uhd, col, leds)
}

fn display_metrics(mut uhd: &mut UnicornHatHd, metrics: Vec<MetricType>) -> Result<(), Error> {
    let mut current_column = 0;
    for metric in metrics {
        match metric {
            MetricType::ColumnRatio { width: width @ _, values: values @ _, colors: colors @ _ } => {
                for i in 0..width {
                    fill_column_ratio(&mut uhd, current_column, values.clone(), colors.clone())?;
                    current_column += 1;
                }
            },
            MetricType::ColumnCount { width: _, value: _ } => unimplemented!(),
        }
    }
    uhd.display()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_of_leds() {
        let original_vals = vec![0, 1, 3];
        let expected_counts = vec![0, 4, 12];
        let result = vector_of_leds(original_vals.clone());
        assert_eq!(
            result,
            expected_counts,
            "vector_of_leds({:?}) -> {:?} (Expected: {:?})",
            original_vals,
            result,
            expected_counts
        );

        let original_vals = vec![62, 6, 4];
        let expected_counts = vec![14, 1, 1];
        let result = vector_of_leds(original_vals.clone());
        assert_eq!(
            result,
            expected_counts,
            "vector_of_leds({:?}) -> {:?} (Expected: {:?})",
            original_vals,
            result,
            expected_counts
        );

        let original_vals = vec![14, 2, 3];
        let expected_counts = vec![12, 1, 3];
        let result = vector_of_leds(original_vals.clone());
        assert_eq!(
            result,
            expected_counts,
            "vector_of_leds({:?}) -> {:?} (Expected: {:?})",
            original_vals,
            result,
            expected_counts
        );

        let original_vals = vec![0, 5];
        let expected_counts = vec![0, 16];
        let result = vector_of_leds(original_vals.clone());
        assert_eq!(
            result,
            expected_counts,
            "vector_of_leds({:?}) -> {:?} (Expected: {:?})",
            original_vals,
            result,
            expected_counts
        );

        let original_vals = vec![47];
        let expected_counts = vec![16];
        let result = vector_of_leds(original_vals.clone());
        assert_eq!(
            result,
            expected_counts,
            "vector_of_leds({:?}) -> {:?} (Expected: {:?})",
            original_vals,
            result,
            expected_counts
        );

        let original_vals = vec![0];
        let expected_counts = vec![0];
        let result = vector_of_leds(original_vals.clone());
        assert_eq!(
            result,
            expected_counts,
            "vector_of_leds({:?}) -> {:?} (Expected: {:?})",
            original_vals,
            result,
            expected_counts
        );

        let original_vals = vec![0, 0];
        let expected_counts = vec![0, 0];
        let result = vector_of_leds(original_vals.clone());
        assert_eq!(
            result,
            expected_counts,
            "vector_of_leds({:?}) -> {:?} (Expected: {:?})",
            original_vals,
            result,
            expected_counts
        );
    }
}
