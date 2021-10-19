use std::path::PathBuf;

pub enum Command {
    Apply,
    Delete,
}

pub enum Resource {
    File(PathBuf),
    Namespace(String),
}

pub struct KubeCtlRunner {
    command: Command,
    resource: Resource,
}

impl KubeCtlRunner {
    pub fn new(cmd: Command, res: Resource) -> Self {
        Self {
            command: cmd,
            resource: res,
        }
    }

    pub async fn run(self) -> Result<(), std::io::Error> {
        let cmd_string = match self.command {
            Command::Apply => "apply",
            Command::Delete => "delete",
        };

        let res_args = match &self.resource {
            Resource::File(path) => {
                let path_string = path.to_str().unwrap();
                ["-f", path_string]
            }
            Resource::Namespace(namespace) => ["namespace", &namespace],
        };

        let mut handle = tokio::process::Command::new("kubectl")
            .arg(cmd_string)
            .args(res_args)
            .spawn()?;

        handle.wait().await?;

        Ok(())
    }
}
