use chrono::prelude::{DateTime, Utc};
use config::{ConfigError, Config, File};
use failure::Error;

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
    pub since: Option<String>,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut c = Config::new();
        c.merge(File::with_name(".gh-univiewer").required(true))?;

        c.try_into()
    }
}

impl RepositorySettings {
    pub fn closed_since_date(&self) -> DateTime<Utc> {
        Utc::now()
    }
}
