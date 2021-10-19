use std::path::PathBuf;

pub enum Command {
    Apply { resource: Resource },
    Delete { resource: Resource },
    List { resource: String },
}

pub enum Resource {
    File(PathBuf),
    Specific(String, String),
}

impl Resource {
    pub fn to_args(&self) -> Vec<String> {
        match &self {
            Self::File(path) => vec!["-f".to_owned(), path.to_str().unwrap().to_string()],
            Self::Specific(ty, name) => vec![ty.to_owned(), name.to_owned()],
        }
    }
}

pub struct KubeCtlRunner {
    command: Command,
}

impl KubeCtlRunner {
    pub fn new(cmd: Command) -> Self {
        Self { command: cmd }
    }

    pub async fn run(self) -> Result<(), std::io::Error> {
        let mut cmd = tokio::process::Command::new("kubectl");

        match &self.command {
            Command::Apply { resource } => {
                cmd.arg("apply");
                cmd.args(resource.to_args());
            }
            Command::Delete { resource } => {
                cmd.arg("delete");
                cmd.args(resource.to_args());
            }
            Command::List { resource } => {
                cmd.arg("get");
                cmd.arg(resource);
            }
        };

        let mut handle = cmd.spawn()?;

        handle.wait().await?;

        Ok(())
    }
}
