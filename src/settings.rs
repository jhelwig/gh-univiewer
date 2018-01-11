use std::str::FromStr;

use chrono::prelude::{DateTime, Utc};
use chrono::Duration;
use config::{ConfigError, Config, File};
use failure::Error;
use serde::{self, Deserialize, Deserializer};

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub github_token: String,
    pub repositories: Vec<RepositorySettings>,
}

#[derive(Debug, Deserialize)]
pub struct RepositorySettings {
    pub user: String,
    pub name: String,
    pub labels: Option<Vec<String>>,
    pub since: Option<Since>,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut c = Config::new();
        c.merge(File::with_name(".gh-univiewer").required(true))?;

        c.try_into()
    }
}

impl RepositorySettings {
    pub fn closed_since_date(&self) -> Option<DateTime<Utc>> {
        if let Some(ref s) = self.since {
            match *s {
                Since { quantity: q @ _, unit: SinceSpan::Day } => {
                    return Some(Utc::now().checked_sub_signed(Duration::days(q as i64)).unwrap())
                },
                Since { quantity: q @ _, unit: SinceSpan::Week } => {
                    return Some(Utc::now().checked_sub_signed(Duration::weeks(q as i64)).unwrap())
                },
            }
        };

        None
    }
}

#[derive(Debug, PartialEq)]
pub struct Since {
    quantity: u64,
    unit: SinceSpan,
}

impl FromStr for Since {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tokens: Vec<&str> = s.split_whitespace().collect();
        if tokens.len() != 2 {
            return Err(format_err!("Value for 'since' should be exactly two words (found {} words in '{}')", tokens.len(), s));
        }

        let quantity = tokens[0].parse::<u64>()?;
        let interval = tokens[1].parse::<SinceSpan>()?;

        Ok(Since { quantity: quantity, unit: interval, })
    }
}

impl<'de> Deserialize<'de> for Since {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        let val = String::deserialize(deserializer)?;
        val.parse::<Since>().map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, PartialEq)]
pub enum SinceSpan {
    Day,
    Week,
}

impl FromStr for SinceSpan {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "day"  | "days"  => Ok(SinceSpan::Day),
            "week" | "weeks" => Ok(SinceSpan::Week),
            _ => Err(format_err!("'{}' is not a recognized interval", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use chrono::Duration;
    use spectral::prelude::*;
    use spectral::{AssertionFailure, Spec};

    trait SinceTemporality {
        fn is_temporally_close_to(&mut self, expected: DateTime<Utc>);
    }

    impl<'s> SinceTemporality for Spec<'s, DateTime<Utc>> {
        fn is_temporally_close_to(&mut self, expected: DateTime<Utc>) {
            let subject = self.subject;
            let delta = expected.signed_duration_since(subject.clone());
            if delta.num_seconds().abs() > 2 {
                AssertionFailure::from_spec(self)
                    .with_expected(format!("'{}' to be within 2 seconds of '{}'", subject, expected))
                    .with_actual(format!("{} seconds difference", delta.num_seconds()))
                    .fail();
            }
        }
    }

    trait AssertSinceSpan {
        fn is_since(&mut self, expected_quantity: u64, expected_unit: SinceSpan);
    }

    impl<'s> AssertSinceSpan for Spec<'s, Since> {
        fn is_since(&mut self, expected_quantity: u64, expected_unit: SinceSpan) {
            let subject = self.subject;
            let expected = Since { quantity: expected_quantity, unit: expected_unit };
            if *subject != expected {
                AssertionFailure::from_spec(self)
                    .with_expected(format!("{:?}", expected))
                    .with_actual(format!("{:?}", subject))
                    .fail();
            }
        }
    }

    #[test]
    fn parse_since_wrong_number_of_words() {
        let parsed = "1".parse::<Since>();
        asserting!("parse since requires more than one word").that(&parsed).is_err();

        let parsed = "1 too many".parse::<Since>();
        asserting!("parse since requires exactly two words").that(&parsed).is_err();
    }

    #[test]
    fn parse_since_first_word_not_a_number() {
        let parsed = "bob weeks".parse::<Since>();
        asserting!("parse since requires that the first word is a number").that(&parsed).is_err();
    }

    #[test]
    fn parse_since_second_word_not_a_recognized_interval() {
        let parsed = "1 bob".parse::<Since>();
        asserting!("parse since requires that the second word is a recognized interval").that(&parsed).is_err();
    }

    #[test]
    fn parse_since_days() {
        let parsed = "1 day".parse::<Since>();
        asserting!("parse_since(\"1 day\")").that(&parsed).is_ok().is_since(1, SinceSpan::Day);

        let parsed = "2 days".parse::<Since>();
        asserting!("parse_since(\"2 days\")").that(&parsed).is_ok().is_since(2, SinceSpan::Day);

        let parsed = "12 days".parse::<Since>();
        asserting!("parse_since(\"12 days\")").that(&parsed).is_ok().is_since(12, SinceSpan::Day);
    }

    #[test]
    fn parse_since_weeks() {
        let parsed = "1 week".parse::<Since>();
        asserting!("parse_since(\"1 week\")").that(&parsed).is_ok().is_since(1, SinceSpan::Week);

        let parsed = "2 weeks".parse::<Since>();
        asserting!("parse_since(\"2 weeks\")").that(&parsed).is_ok().is_since(2, SinceSpan::Week);

        let parsed = "12 weeks".parse::<Since>();
        asserting!("parse_since(\"12 weeks\")").that(&parsed).is_ok().is_since(12, SinceSpan::Week);
    }

    fn repository_settings_since(since: &str) -> RepositorySettings {
        RepositorySettings {
            user: "a".to_string(),
            name: "b".to_string(),
            labels: None,
            since: Some(since.parse::<Since>().unwrap()),
        }
    }

    #[test]
    fn closed_since_date_days() {
        let closed_since_one_day_ago = repository_settings_since("1 day").closed_since_date().unwrap();
        let one_day_ago = Utc::now().checked_sub_signed(Duration::days(1)).unwrap();
        asserting!("1 day").that(&closed_since_one_day_ago).is_temporally_close_to(one_day_ago);

        let closed_since_two_days_ago = repository_settings_since("2 days").closed_since_date().unwrap();
        let two_days_ago = Utc::now().checked_sub_signed(Duration::days(2)).unwrap();
        asserting!("2 days").that(&closed_since_two_days_ago).is_temporally_close_to(two_days_ago);

        let closed_since_twelve_days_ago = repository_settings_since("12 days").closed_since_date().unwrap();
        let twelve_days_ago = Utc::now().checked_sub_signed(Duration::days(12)).unwrap();
        asserting!("12 days").that(&closed_since_twelve_days_ago).is_temporally_close_to(twelve_days_ago);
    }

    #[test]
    fn closed_since_date_weeks() {
        let closed_since_one_week_ago = repository_settings_since("1 week").closed_since_date().unwrap();
        let one_week_ago = Utc::now().checked_sub_signed(Duration::weeks(1)).unwrap();
        asserting!("1 week").that(&closed_since_one_week_ago).is_temporally_close_to(one_week_ago);

        let closed_since_two_weeks_ago = repository_settings_since("2 weeks").closed_since_date().unwrap();
        let two_weeks_ago = Utc::now().checked_sub_signed(Duration::weeks(2)).unwrap();
        asserting!("2 weeks").that(&closed_since_two_weeks_ago).is_temporally_close_to(two_weeks_ago);

        let closed_since_twelve_weeks_ago = repository_settings_since("12 weeks").closed_since_date().unwrap();
        let twelve_weeks_ago = Utc::now().checked_sub_signed(Duration::weeks(12)).unwrap();
        asserting!("12 weeks").that(&closed_since_twelve_weeks_ago).is_temporally_close_to(twelve_weeks_ago);
    }
}