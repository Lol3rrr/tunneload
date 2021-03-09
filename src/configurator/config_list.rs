use std::fmt::Debug;

use crate::general::Shared;

pub trait ConfigItem {
    /// Returns the Name of the ConfigItem
    fn name(&self) -> &str;
}

#[derive(Debug)]
pub struct ConfigList<C>
where
    C: ConfigItem + Debug,
{
    entries: std::sync::Arc<std::sync::Mutex<std::collections::BTreeMap<String, Shared<C>>>>,
}

impl<C> Clone for ConfigList<C>
where
    C: ConfigItem + Debug,
{
    fn clone(&self) -> Self {
        Self {
            entries: self.entries.clone(),
        }
    }
}

impl<C> ConfigList<C>
where
    C: ConfigItem + Debug,
{
    pub fn new() -> Self {
        Self {
            entries: std::sync::Arc::new(std::sync::Mutex::new(std::collections::BTreeMap::new())),
        }
    }

    pub fn set(&self, n_conf: C) -> usize {
        let mut inner = self.entries.lock().unwrap();

        inner.insert(n_conf.name().to_owned(), Shared::new(n_conf));
        inner.len()
    }

    pub fn get<S>(&self, name: S) -> Option<Shared<C>>
    where
        S: AsRef<str>,
    {
        let inner = self.entries.lock().unwrap();

        inner.get(name.as_ref()).and_then(|tmp| Some(tmp.clone()))
    }

    pub fn remove<S>(&self, name: S) -> usize
    where
        S: AsRef<str>,
    {
        let mut inner = self.entries.lock().unwrap();

        inner.remove(name.as_ref());
        inner.len()
    }
}

impl<C> Default for ConfigList<C>
where
    C: ConfigItem + Debug,
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct MockConfigItem {
        name: String,
    }
    impl ConfigItem for MockConfigItem {
        fn name(&self) -> &str {
            &self.name
        }
    }

    #[test]
    fn set_entry() {
        let tmp_list: ConfigList<MockConfigItem> = ConfigList::new();

        tmp_list.set(MockConfigItem {
            name: "test-name".to_owned(),
        });

        let mut result_map: std::collections::BTreeMap<String, Shared<MockConfigItem>> =
            std::collections::BTreeMap::new();
        result_map.insert(
            "test-name".to_owned(),
            Shared::new(MockConfigItem {
                name: "test-name".to_owned(),
            }),
        );
        assert_eq!(result_map, tmp_list.entries.lock().unwrap().clone());
    }

    #[test]
    fn set_get_entry_valid() {
        let tmp_list: ConfigList<MockConfigItem> = ConfigList::new();

        tmp_list.set(MockConfigItem {
            name: "test-name".to_owned(),
        });

        assert_eq!(
            Some(Shared::new(MockConfigItem {
                name: "test-name".to_owned()
            })),
            tmp_list.get("test-name")
        );
    }
    #[test]
    fn set_get_entry_invalid() {
        let tmp_list: ConfigList<MockConfigItem> = ConfigList::new();

        tmp_list.set(MockConfigItem {
            name: "test-name".to_owned(),
        });

        assert_eq!(None, tmp_list.get("other-name"));
    }

    #[test]
    fn set_remove_entry_valid() {
        let tmp_list: ConfigList<MockConfigItem> = ConfigList::new();

        tmp_list.set(MockConfigItem {
            name: "test-name".to_owned(),
        });

        tmp_list.remove("test-name");

        assert_eq!(
            std::collections::BTreeMap::new(),
            tmp_list.entries.lock().unwrap().clone()
        );
    }
    #[test]
    fn set_remove_entry_invalid() {
        let tmp_list: ConfigList<MockConfigItem> = ConfigList::new();

        tmp_list.set(MockConfigItem {
            name: "test-name".to_owned(),
        });

        tmp_list.remove("other-name");

        let mut result_map: std::collections::BTreeMap<String, Shared<MockConfigItem>> =
            std::collections::BTreeMap::new();
        result_map.insert(
            "test-name".to_owned(),
            Shared::new(MockConfigItem {
                name: "test-name".to_owned(),
            }),
        );
        assert_eq!(result_map, tmp_list.entries.lock().unwrap().clone());
    }
}
