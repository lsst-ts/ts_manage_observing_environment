use crate::error::ObsEnvError;
use git2::{build::CheckoutBuilder, DescribeOptions, Error, FetchOptions, Repository};
use regex::Regex;
use std::{
    collections::BTreeMap,
    fs::{create_dir, File},
    io::{BufRead, BufReader},
    path::Path,
};

const REPO_VERSION_REGEXP: &str = r"(?P<name>[a-zA-Z0-9_]*)=(?P<version>[a-zA-Z0-9._]*)";
const VALID_VERSION: &str = r"^(?P<major>[0-9]*)\.(?P<minor>[0-9]*)\.(?P<patch>[0-9]*)";

pub struct ObservingEnvironment {
    /// List of repositories that belong to the observing environment.
    repositories: BTreeMap<String, String>,
    /// Organzation url for the base env sourve repository
    base_env_source_org: String,
    /// Repository with the base environment version definitions
    base_env_source_repo: String,
    /// File path in the base environment version definitions repository
    /// with the version information
    base_env_def_file: String,
    /// Location where the repositories should be placed in the host.
    destination: String,
}

impl Default for ObservingEnvironment {
    fn default() -> ObservingEnvironment {
        ObservingEnvironment {
            repositories: BTreeMap::from_iter([
                (
                    "atmospec".to_owned(),
                    r"https://github.com/lsst/".to_owned(),
                ),
                ("cwfs".to_owned(), r"https://github.com/lsst-ts/".to_owned()),
                (
                    "Spectractor".to_owned(),
                    r"https://github.com/lsst-dm/".to_owned(),
                ),
                (
                    "summit_extras".to_owned(),
                    r"https://github.com/lsst-sitcom/".to_owned(),
                ),
                (
                    "summit_utils".to_owned(),
                    r"https://github.com/lsst-sitcom/".to_owned(),
                ),
                (
                    "ts_config_attcs".to_owned(),
                    r"https://github.com/lsst-ts/".to_owned(),
                ),
                (
                    "ts_config_ocs".to_owned(),
                    r"https://github.com/lsst-ts/".to_owned(),
                ),
                (
                    "ts_externalscripts".to_owned(),
                    r"https://github.com/lsst-ts/".to_owned(),
                ),
                (
                    "ts_observatory_control".to_owned(),
                    r"https://github.com/lsst-ts/".to_owned(),
                ),
                (
                    "ts_observing_utilities".to_owned(),
                    r"https://github.com/lsst-ts/".to_owned(),
                ),
                (
                    "ts_standardscripts".to_owned(),
                    r"https://github.com/lsst-ts/".to_owned(),
                ),
                (
                    "ts_wep".to_owned(),
                    r"https://github.com/lsst-ts/".to_owned(),
                ),
            ]),
            base_env_source_org: r"https://github.com/lsst-ts/".to_owned(),
            base_env_source_repo: "ts_cycle_build".to_owned(),
            base_env_def_file: "cycle/cycle.env".to_owned(),
            destination: "/obs-env".to_owned(),
        }
    }
}

impl ObservingEnvironment {
    pub fn with_destination(dest: &str) -> ObservingEnvironment {
        ObservingEnvironment {
            destination: dest.to_owned(),
            ..Default::default()
        }
    }

    pub fn summarize(&self) -> String {
        format!(
            "Obs. Env. Path: {}.\nNumber of repositories: {}",
            self.destination,
            self.repositories.len()
        )
    }
    /// Check if destination directory exists.
    pub fn create_path(&self) -> Result<(), std::io::Error> {
        let destination = Path::new(&self.destination);

        if !destination.exists() {
            create_dir(&self.destination)
        } else {
            Ok(())
        }
    }

    /// Clone repositories into the environment path.
    pub fn clone_repositories(&self) -> Vec<Result<Repository, Error>> {
        self.repositories
            .iter()
            .filter(|(repo_name, _)| !Path::new(&self.destination).join(repo_name).exists())
            .map(|(repo_name, org)| {
                log::debug!("Cloning: {repo_name}");
                Repository::clone(
                    &format!("{}/{}", org, repo_name),
                    Path::new(&self.destination).join(repo_name),
                )
            })
            .collect()
    }

    /// Reset all repositories to their official version.
    pub fn reset_base_environment(&self, base_env_branch: &str) -> Result<(), Vec<ObsEnvError>> {
        match self.get_base_env_versions(base_env_branch) {
            Ok(obs_env_versions) => {
                let reset_result: Vec<ObsEnvError> = obs_env_versions
                    .into_iter()
                    .map(|(repo, version)| self.reset_index_to_version(&repo, &version))
                    .into_iter()
                    .filter(|result| result.is_err())
                    .map(|err| err.unwrap_err())
                    .collect();

                if reset_result.is_empty() {
                    Ok(())
                } else {
                    Err(reset_result)
                }
            }
            Err(err_get_base_env_versions) => Err(vec![err_get_base_env_versions]),
        }
    }

    /// Checkout branch on specified repository.
    pub fn checkout_branch(&self, repo_name: &str, branch_name: &str) -> Result<(), ObsEnvError> {
        if self.repositories.contains_key(repo_name) {
            match Repository::open(Path::new(&self.destination).join(repo_name)) {
                Ok(repository) => {
                    let spec = format!("refs/remotes/origin/{branch_name}");
                    match checkout_branch(&repository, &spec, branch_name) {
                        Ok(_) => Ok(()),
                        Err(error) => Err(ObsEnvError::GIT(format!(
                            "Failed to checkout branch {branch_name}: {}",
                            error.message()
                        ))),
                    }
                }
                Err(error) => Err(ObsEnvError::GIT(format!(
                    "Failed to open repository {repo_name}: {}",
                    error.message()
                ))),
            }
        } else {
            Err(ObsEnvError::ERROR(format!(
                "Repository {repo_name} not in the list of managed repositories."
            )))
        }
    }

    /// Update the base environment source file.
    fn update_base_env_source(&self, base_env_branch: &str) -> Result<(), Error> {
        let base_env_source_repo = self.get_base_env_source_repo()?;

        let mut remote = base_env_source_repo.find_remote("origin")?;

        remote.fetch(&[base_env_branch], None, None)?;

        let branch_main_remote = base_env_source_repo.find_branch(
            &format!("/origin/{base_env_branch}"),
            git2::BranchType::Remote,
        )?;

        let commit = branch_main_remote.get().peel_to_commit()?;

        let object = commit.as_object();

        base_env_source_repo.reset(object, git2::ResetType::Hard, None)
    }

    fn get_base_env_source_repo(&self) -> Result<Repository, Error> {
        let base_env_source_path = Path::new(&self.destination).join(&self.base_env_source_repo);

        if !base_env_source_path.exists() {
            // need to clone base env source repo
            Repository::clone(
                &format!("{}/{}", self.base_env_source_org, self.base_env_source_repo),
                base_env_source_path,
            )
        } else {
            Repository::open(base_env_source_path.as_path())
        }
    }

    /// Get base versions of all the packages.
    ///
    /// This method will parse the base_env_def_file (e.g. cycle/cycle.env) to
    /// get the versions of the base env packages.
    pub fn get_base_env_versions(
        &self,
        base_env_branch: &str,
    ) -> Result<BTreeMap<String, String>, ObsEnvError> {
        match self.update_base_env_source(base_env_branch) {
            Ok(_) => {
                match self.load_base_env_def_file() {
                    Ok(base_env_def) => {
                        let base_env_versions: Vec<Option<&String>> = self
                            .repositories
                            .iter()
                            .map(|(repo_name, _)| {
                                base_env_def.iter().find(|line| line.starts_with(repo_name))
                            })
                            .collect();
                        // This should never fail because we know REPO_VERSION_REGEXP is
                        // valid.
                        let regex = Regex::new(REPO_VERSION_REGEXP).unwrap();
                        Ok(base_env_versions
                            .into_iter()
                            .filter(|name_version| name_version.is_some())
                            .map(|name_version| regex.captures(name_version.unwrap()))
                            .filter(|captured_name_version| captured_name_version.is_some())
                            .map(|captured_name_version| {
                                if let Some(captured_name_version) = captured_name_version {
                                    (
                                        captured_name_version["name"].to_owned(),
                                        captured_name_version["version"].to_owned(),
                                    )
                                } else {
                                    panic!("Could not read captured name/version");
                                }
                            })
                            .collect())
                    }
                    Err(obs_env_err) => Err(obs_env_err),
                }
            }
            Err(obs_env_err) => Err(ObsEnvError::ERROR(obs_env_err.to_string())),
        }
    }

    /// Get current package versions.
    pub fn get_current_env_versions(&self) -> BTreeMap<String, Result<String, ObsEnvError>> {
        self.repositories
            .iter()
            .map(|(repo_name, _)| (repo_name.to_owned(), self.get_current_version(repo_name)))
            .collect()
    }

    fn get_current_version(&self, repo_name: &str) -> Result<String, ObsEnvError> {
        match Repository::open(Path::new(&self.destination).join(repo_name)) {
            Ok(repository) => {
                let mut opts = DescribeOptions::new();

                match repository.describe(&opts.show_commit_oid_as_fallback(true)) {
                    Ok(description) => match description.format(None) {
                        Ok(description) => Ok(description),
                        Err(error) => Err(ObsEnvError::GIT(format!(
                            "Error describing {repo_name}: {}",
                            error.message()
                        ))),
                    },
                    Err(error) => Err(ObsEnvError::GIT(format!(
                        "Can't retrieve {repo_name} HEAD: {}",
                        error.message()
                    ))),
                }
            }
            Err(error) => Err(ObsEnvError::GIT(format!(
                "Failed to open repository {repo_name}: {}",
                error.message()
            ))),
        }
    }

    /// Read base_env_def_file and return the content.
    fn load_base_env_def_file(&self) -> Result<Vec<String>, ObsEnvError> {
        match File::open(
            Path::new(&self.destination)
                .join(&self.base_env_source_repo)
                .join(&self.base_env_def_file),
        ) {
            Ok(file) => {
                Ok(BufReader::new(file)
                    .lines()
                    .into_iter()
                    .filter(|line| line.is_ok())
                    .map(|line| line.unwrap())
                    .collect())
                // Note it is safe to unwrap inside the map because of the filter.
            }
            Err(error) => Err(ObsEnvError::ERROR(error.to_string())),
        }
    }

    /// Reset repo index to the provided version.
    ///
    /// The version string must have the following format <X>.<Y>.<Z><RT><RN>,
    /// where:
    ///     X, is the major version number.
    ///     Y, is the minor version number.
    ///     Z, is the patch version number.
    ///     RT, is the type of the release. This should be empty if this is an
    ///         official release or:
    ///         a, alpha release.
    ///         b, beta release.
    ///         rc, release candidate.
    ///     RN, is the major version number. If RT is provided than a release
    ///         type number can also be provided.
    ///
    /// Example valid release strings:
    ///     0.1.0
    ///     1.2.3
    ///     1.0.0a, alpha release with no release number.
    ///     1.0.0a1, alpha release with release number 1.
    ///     1.0.0b5, beta release with release number 5.
    ///     1.0.0rc3, release candidate with release number 3.
    fn reset_index_to_version(&self, repo: &str, version: &str) -> Result<(), ObsEnvError> {
        log::debug!("Resetting {repo} to {version}");
        if let Ok(repository) = Repository::open(Path::new(&self.destination).join(repo)) {
            let tag = ObservingEnvironment::expand_version_to_tag(version);

            match ObservingEnvironment::checkout_tag_or_branch(repository, &tag, version) {
                Ok(()) => Ok(()),
                Err(error) => Err(ObsEnvError::GIT(format!(
                    "Could not checkout tag or branch for {repo}@{tag}[{version}]: {}",
                    error.message().to_owned()
                ))),
            }
        } else {
            Err(ObsEnvError::GIT(format!(
                "Failed to open repository: {repo}"
            )))
        }
    }

    /// Expands version string into a tag, following the format adopted by
    /// TSSW.
    fn expand_version_to_tag(version: &str) -> String {
        let version_regex = Regex::new(VALID_VERSION).unwrap();

        if version_regex.is_match(version) {
            format!("v{version}")
                .replace("a", ".alpha.")
                .replace("b", ".beta.")
                .replace("rc", ".rc.")
        } else {
            version.to_owned()
        }
    }

    fn checkout_tag_or_branch(
        repository: Repository,
        tag: &str,
        version: &str,
    ) -> Result<(), Error> {
        log::trace!("Fetching...");
        let mut fetch_options = FetchOptions::new();
        fetch_options.download_tags(git2::AutotagOption::All);

        repository
            .find_remote("origin")?
            .fetch(&[""], Some(&mut fetch_options), None)?;

        // Try to find the tag first
        let spec = "refs/tags/".to_owned() + tag;
        log::trace!("Checkout spec {spec}");
        match repository.revparse_single(&spec) {
            Ok(object) => checkout_tag(&repository, version, object, &spec),
            Err(_) => {
                // Fallback to try finding a branch
                let spec = "refs/remotes/origin/".to_owned() + tag;
                log::trace!("Failed to check tag, trying it as a branch: {spec}");
                checkout_branch(&repository, &spec, version)
            }
        }
    }
}

fn checkout_tag(
    repository: &Repository,
    version: &str,
    object: git2::Object,
    spec: &str,
) -> Result<(), Error> {
    repository.branch(version, &object.peel_to_commit().unwrap(), true)?;
    repository.set_head(spec)?;
    let mut checkout_build = CheckoutBuilder::new();
    repository.reset(&object, git2::ResetType::Hard, Some(checkout_build.force()))?;
    Ok(())
}

fn checkout_branch(repository: &Repository, spec: &str, branch_name: &str) -> Result<(), Error> {
    repository
        .find_remote("origin")?
        .fetch(&[branch_name], None, None)?;

    let object = repository.revparse_single(spec)?;

    repository.branch(branch_name, &object.peel_to_commit().unwrap(), true)?;
    repository.set_head(&spec)?;
    let mut checkout_build = CheckoutBuilder::new();
    repository.reset(&object, git2::ResetType::Hard, Some(checkout_build.force()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use regex::Regex;

    use super::{ObservingEnvironment, REPO_VERSION_REGEXP, VALID_VERSION};

    use once_cell::sync::Lazy;
    use std::sync::Mutex;

    static REPO_ACCESS: Lazy<Mutex<()>> = Lazy::new(Mutex::default);

    type TestResult<T = (), E = Box<dyn std::error::Error>> = std::result::Result<T, E>;

    #[test]
    fn test_repo_version_regexp() {
        let regexp = Regex::new(REPO_VERSION_REGEXP).unwrap();

        let repo_version = regexp.captures("ts_unit_test=X.Y.ZaN").unwrap();

        assert_eq!(&repo_version["name"], "ts_unit_test");
        assert_eq!(&repo_version["version"], "X.Y.ZaN");
    }

    #[test]
    fn expand_version_to_tag() {
        assert_eq!(
            ObservingEnvironment::expand_version_to_tag("1.0.0"),
            "v1.0.0"
        );
        assert_eq!(
            ObservingEnvironment::expand_version_to_tag("1.0.0a1"),
            "v1.0.0.alpha.1"
        );
        assert_eq!(
            ObservingEnvironment::expand_version_to_tag("1.0.0b1"),
            "v1.0.0.beta.1"
        );
        assert_eq!(
            ObservingEnvironment::expand_version_to_tag("1.0.0rc1"),
            "v1.0.0.rc.1"
        );
    }

    #[test]
    fn test_update_base_env_source() {
        let _shared = REPO_ACCESS.lock().unwrap();

        let obs_env = ObservingEnvironment::with_destination(".");

        obs_env.update_base_env_source("main").unwrap();

        assert!(Path::new(&obs_env.destination)
            .join(obs_env.base_env_source_repo)
            .exists())
    }

    #[test]
    fn test_get_base_env_versions() {
        let _shared = REPO_ACCESS.lock().unwrap();
        let obs_env = ObservingEnvironment::with_destination(".");

        let base_env_versions = obs_env.get_base_env_versions("main").unwrap();

        for (repo, _) in obs_env.repositories {
            assert!(base_env_versions.contains_key(&repo))
        }
    }

    #[test]
    fn test_is_valid_version() {
        let version_regex = Regex::new(VALID_VERSION).unwrap();

        assert!(version_regex.is_match("1.2.3"));
        assert!(version_regex.is_match("10.200.300"));
        assert!(version_regex.is_match("1.20.3a1"));
        assert!(version_regex.is_match("1.20.3b1"));
        assert!(version_regex.is_match("1.20.3rc1"));
        assert!(!version_regex.is_match("w.2023.13"));
        assert!(!version_regex.is_match("main"));
        assert!(!version_regex.is_match("develop"));
        assert!(!version_regex.is_match("ticket/DM-12345"));
    }
}
