use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use dirs;
use failure::Fail;
use serde::Deserialize;
use serde::Serialize;

use crate::types::ResultDynError;

#[derive(Debug, Fail)]
pub enum ProjectConfigError {
  #[fail(
    display = "Project config {} does not exist, please check config or create it",
    name
  )]
  ProjectConfigDoesNotExist { name: String },
}

#[derive(Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProjectConfig {
  pub name: String,
  pub db_uri: String,
}

#[derive(Serialize, Deserialize)]
pub struct JabConfig {
  pub projects: HashMap<String, ProjectConfig>,
}

impl JabConfig {
  pub fn read() -> ResultDynError<JabConfig> {
    let config_path = JabConfig::get_path();
    let config_str = String::from(fs::read_to_string(config_path)?);
    let config: JabConfig = serde_json::from_str(&config_str)?;

    return Ok(config);
  }

  pub fn persist(config: &JabConfig) -> ResultDynError<()> {
    let config_path = JabConfig::get_path();

    let config_str = serde_json::to_string_pretty(&config)?;
    fs::write(config_path, config_str)?;

    return Ok(());
  }

  pub fn get_path() -> PathBuf {
    return get_jab_dir().join("config");
  }

  pub fn empty_config_str() -> String {
    let config = JabConfig {
      projects: HashMap::new(),
    };

    return serde_json::to_string_pretty(&config).unwrap();
  }
}

impl JabConfig {
  pub fn register_project_config(&mut self, project_config: ProjectConfig) {
    self
      .projects
      .insert(project_config.name.clone(), project_config);
  }

  pub fn project_config(&self, name: &str) -> ResultDynError<&ProjectConfig> {
    return self.projects.get(name).ok_or(
      ProjectConfigError::ProjectConfigDoesNotExist {
        name: String::from(name),
      }
      .into(),
    );
  }
}

pub fn get_jab_dir() -> PathBuf {
  let project_dir = dirs::home_dir().unwrap();

  return project_dir.join(".jab");
}
