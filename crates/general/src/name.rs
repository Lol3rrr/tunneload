use std::{fmt::Display, hash::Hash};

use serde::Serialize;

/// A single Name for a Resource, which consists of a user given Name and a Groupt to distinguish
/// between two Resources that may have the same Name but come from two different Systems/Sources
#[derive(Debug, Eq, Clone, Serialize)]
pub struct Name {
    name: String,
    group: Group,
}

impl Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}@{}", self.name, self.group)
    }
}

impl PartialEq for Name {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.group == other.group
    }
}

impl Hash for Name {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let entire_name = self.to_string();

        entire_name.hash(state);
    }
}

/// The Group to which a Resource belongs or was loaded from, like Kubernetes or a File
#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
pub enum Group {
    /// The Kubernetes Group is used to describe everything coming from Kubernetes regardless
    /// of the actual underlying Type, like ingress or CRD
    Kubernetes {
        /// The Namespace of the Resource, this is the main way to distinguish two Resources with
        /// the same Name in kubernetes
        namespace: String,
    },
    /// The File Group is used to describe everything coming from Files, right now there is no way
    /// to distinguish between different Files and they are all in the same Group
    File {},
    /// The Internal Group is used to describe everything coming from some internal
    /// Tunneload specific Part, like the Dashboard
    Internal,
}

impl Display for Group {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::Kubernetes { namespace } => write!(f, "k8s@{}", namespace),
            Self::File {} => write!(f, "file"),
            Self::Internal => write!(f, "internal"),
        }
    }
}

impl Name {
    /// Creates a new Name based on the given Configuration
    pub fn new<S>(name: S, group: Group) -> Self
    where
        S: Into<String>,
    {
        Self {
            name: name.into(),
            group,
        }
    }

    /// The Name of the Resource without the Group or any other Specific Data
    ///
    /// # NOTE
    /// This is likely not unique and should therefore also not be used as an identify
    /// only for displaying purposes
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Parses a Resource string into its Name representation and using the fallback as a way to
    /// get the Rule implicitly, if it was not specified in the Resource-String
    pub fn parse<F>(raw_name: &str, fallback: F) -> Self
    where
        F: FnOnce() -> Group,
    {
        match raw_name.split_once('@') {
            Some((name, raw_group)) => {
                let group = match Group::parse(raw_group) {
                    Some(g) => g,
                    None => fallback(),
                };

                Self {
                    name: name.to_owned(),
                    group,
                }
            }
            None => {
                let name = raw_name;
                let group = fallback();

                Self {
                    name: name.to_owned(),
                    group,
                }
            }
        }
    }
}

impl Group {
    pub(crate) fn parse(raw_group: &str) -> Option<Self> {
        match raw_group.split_once('@') {
            Some((group, specific)) => match group {
                "k8s" => Some(Self::Kubernetes {
                    namespace: specific.to_owned(),
                }),
                _ => None,
            },
            None => match raw_group {
                "file" => Some(Self::File {}),
                "internal" => Some(Self::Internal),
                _ => None,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn k8s_everything_provided() {
        let input = "testing@k8s@test-ns";
        let expected = Name {
            name: "testing".to_owned(),
            group: Group::Kubernetes {
                namespace: "test-ns".to_string(),
            },
        };

        let result = Name::parse(input, || {
            panic!("The Fallback should not be called because we have all the Data we need");
        });

        assert_eq!(expected, result);
    }
    #[test]
    fn k8s_without_namespace() {
        let input = "testing@k8s";
        let expected = Name {
            name: "testing".to_string(),
            group: Group::Kubernetes {
                namespace: "default".to_string(),
            },
        };

        let result = Name::parse(input, || Group::Kubernetes {
            namespace: "default".to_string(),
        });

        assert_eq!(expected, result);
    }

    #[test]
    fn file_only_delimiter() {
        let input = "testing@file";
        let expected = Name {
            name: "testing".to_string(),
            group: Group::File {},
        };

        let result = Name::parse(input, || {
            panic!(
                "The Fallback should not be called because 'file' does not need any more specifics"
            )
        });

        assert_eq!(expected, result);
    }

    #[test]
    fn internal() {
        let input = "testing@internal";
        let expected = Name {
            name: "testing".to_string(),
            group: Group::Internal,
        };

        let result = Name::parse(input, || {
            panic!("The Fallback should not be called because 'internal' does not need any more specifics")
        });

        assert_eq!(expected, result);
    }

    #[test]
    fn without_group() {
        let input = "testing";
        let expected = Name {
            name: "testing".to_string(),
            group: Group::Kubernetes {
                namespace: "other".to_string(),
            },
        };

        let result = Name::parse(input, || Group::Kubernetes {
            namespace: "other".to_string(),
        });

        assert_eq!(expected, result);
    }
}
