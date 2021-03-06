use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

use crate::git::CommitIterator;
use crate::git::GitRepo;
use crate::types::ResultDynError;

pub struct CreateInput<'a> {
  pub project_name: &'a str,
  pub project_dir: &'a Path,
  pub db_uri: &'a str,
}

pub struct OpenInput<'a> {
  pub project_dir: &'a Path,
  pub project_name: &'a str,
  pub db_uri: &'a str,
}

pub struct Project {
  name: String,
  project_dir: PathBuf,
  repo_path: PathBuf,
  sql_path: PathBuf,
  db_uri: String,
  repo: GitRepo,
}

impl Project {
  pub fn create(input: &CreateInput) -> ResultDynError<Project> {
    let repo_path = input.project_dir.join(&input.project_name);
    let _repo = GitRepo::upsert(repo_path)?;

    let project = Project::open(&OpenInput {
      project_dir: input.project_dir,
      project_name: input.project_name,
      db_uri: input.db_uri,
    })?;

    return Ok(project);
  }

  pub fn open(input: &OpenInput) -> ResultDynError<Project> {
    let repo_path = input.project_dir.join(&input.project_name);
    let repo = GitRepo::new(repo_path.to_str().unwrap())?;

    // TODO: Validate if project exists
    return Ok(Project {
      db_uri: input.db_uri.into(),
      project_dir: input.project_dir.into(),
      name: input.project_name.into(),
      sql_path: Project::default_sql_path(),
      repo_path,
      repo,
    });
  }

  fn default_sql_path() -> PathBuf {
    return PathBuf::from("dump.sql");
  }
}

impl Project {
  pub fn db_uri(&self) -> &str {
    return &self.db_uri;
  }

  pub fn project_dir(&self) -> &Path {
    return self.project_dir.as_ref();
  }

  pub fn repo_path(&self) -> &Path {
    return self.repo_path.as_ref();
  }

  pub fn sql_path(&self) -> &Path {
    return self.sql_path.as_ref();
  }

  pub fn name(&self) -> &str {
    return &self.name;
  }

  pub fn commit_iterator(&self) -> ResultDynError<CommitIterator> {
    return self.repo.commit_iterator();
  }

  pub fn absolute_sql_path(&self) -> PathBuf {
    return self.repo_path.join(&self.sql_path);
  }

  pub fn commit_dump(&self, message: &str, dump: Vec<u8>) -> ResultDynError<()> {
    log::debug!("Creating project...");
    let repo = GitRepo::new(self.repo_path())?;

    log::debug!("Reading db...");
    self.sync_dump(dump)?;

    log::debug!("Writing state changes...");
    repo.commit_file(self.sql_path(), message)?;

    return Ok(());
  }

  pub fn sync_dump(&self, dump: Vec<u8>) -> io::Result<()> {
    // Update content to file in the repo
    return fs::write(self.absolute_sql_path(), dump);
  }

  pub fn get_dump_at_commit(&self, commit_hash: &str) -> ResultDynError<Vec<u8>> {
    return self
      .repo
      .get_file_content_at_commit(self.sql_path(), commit_hash);
  }

  pub fn get_latest_dump(&self) -> ResultDynError<Vec<u8>> {
    let last_commit_hash = self.repo.last_commit_hash()?;

    return self.get_dump_at_commit(&last_commit_hash);
  }
}
