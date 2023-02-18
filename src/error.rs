#[derive(Clone, Debug)]
pub enum ObsEnvError {
    ERROR(String),
    GIT(String),
}
