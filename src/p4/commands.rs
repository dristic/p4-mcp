#[derive(Debug, Clone)]
pub enum P4Command {
    Status {
        path: Option<String>,
    },
    Sync {
        path: String,
        force: bool,
    },
    Edit {
        files: Vec<String>,
    },
    Add {
        files: Vec<String>,
    },
    Submit {
        description: String,
        files: Option<Vec<String>>,
    },
    Revert {
        files: Vec<String>,
    },
    Opened {
        changelist: Option<String>,
    },
    Changes {
        max: u32,
        path: Option<String>,
    },
}

impl P4Command {
    pub fn to_command_args(&self) -> (String, Vec<String>) {
        match self {
            P4Command::Status { path } => {
                let mut args = vec!["opened".to_string()];
                if let Some(p) = path {
                    args.push(p.clone());
                }
                ("p4".to_string(), args)
            }

            P4Command::Sync { path, force } => {
                let mut args = vec!["sync".to_string()];
                if *force {
                    args.push("-f".to_string());
                }
                args.push(path.clone());
                ("p4".to_string(), args)
            }

            P4Command::Edit { files } => {
                let mut args = vec!["edit".to_string()];
                args.extend(files.clone());
                ("p4".to_string(), args)
            }

            P4Command::Add { files } => {
                let mut args = vec!["add".to_string()];
                args.extend(files.clone());
                ("p4".to_string(), args)
            }

            P4Command::Submit { description, files } => {
                let mut args = vec!["submit".to_string(), "-d".to_string(), description.clone()];
                if let Some(f) = files {
                    args.extend(f.clone());
                }
                ("p4".to_string(), args)
            }

            P4Command::Revert { files } => {
                let mut args = vec!["revert".to_string()];
                args.extend(files.clone());
                ("p4".to_string(), args)
            }

            P4Command::Opened { changelist } => {
                let mut args = vec!["opened".to_string()];
                if let Some(cl) = changelist {
                    args.push("-c".to_string());
                    args.push(cl.clone());
                }
                ("p4".to_string(), args)
            }

            P4Command::Changes { max, path } => {
                let mut args = vec!["changes".to_string(), "-m".to_string(), max.to_string()];
                if let Some(p) = path {
                    args.push(p.clone());
                }
                ("p4".to_string(), args)
            }
        }
    }
}
