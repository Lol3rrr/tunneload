use std::fmt::Debug;

use crate::general::Shared;

pub trait ConfigItem {
    /// Returns the Name of the ConfigItem
    fn name(&self) -> &str;
}
pub trait DefaultConfig {
    fn default_name(name: String) -> Self;
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

        let name = n_conf.name();
        match inner.get(name) {
            Some(data) => {
                data.update(n_conf);
            }
            None => {
                inner.insert(name.to_owned(), Shared::new(n_conf));
            }
        };
        inner.len()
    }

    pub fn get<S>(&self, name: S) -> Option<Shared<C>>
    where
        S: AsRef<str>,
    {
        let inner = self.entries.lock().unwrap();

        inner.get(name.as_ref()).cloned()
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

impl<C> ConfigList<C>
where
    C: ConfigItem + Debug + DefaultConfig,
{
    /// This function either returns the Value for the given
    /// name from the Collection itself, or if it does not
    /// exist, creates a Default-Value for C, inserts that
    /// into the Collection and then returns that default
    /// value
    pub fn get_with_default<S>(&self, name: S) -> Shared<C>
    where
        S: AsRef<str>,
    {
        let mut inner = self.entries.lock().unwrap();

        match inner.get(name.as_ref()) {
            Some(c) => c.clone(),
            None => {
                let owned_name = name.as_ref().to_owned();
                let n_item = Shared::new(C::default_name(owned_name.clone()));
                inner.insert(owned_name, n_item.clone());
                n_item
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct MockConfigItem {
        name: String,
        value: u32,
    }
    impl ConfigItem for MockConfigItem {
        fn name(&self) -> &str {
            &self.name
        }
    }
    impl DefaultConfig for MockConfigItem {
        fn default_name(name: String) -> Self {
            Self { name, value: 0 }
        }
    }

    #[test]
    fn set_entry() {
        let tmp_list: ConfigList<MockConfigItem> = ConfigList::new();

        tmp_list.set(MockConfigItem {
            name: "test-name".to_owned(),
            value: 0,
        });

        let mut result_map: std::collections::BTreeMap<String, Shared<MockConfigItem>> =
            std::collections::BTreeMap::new();
        result_map.insert(
            "test-name".to_owned(),
            Shared::new(MockConfigItem {
                name: "test-name".to_owned(),
                value: 0,
            }),
        );
        assert_eq!(result_map, tmp_list.entries.lock().unwrap().clone());
    }

    #[test]
    fn set_get_entry_valid() {
        let tmp_list: ConfigList<MockConfigItem> = ConfigList::new();

        tmp_list.set(MockConfigItem {
            name: "test-name".to_owned(),
            value: 0,
        });

        assert_eq!(
            Some(Shared::new(MockConfigItem {
                name: "test-name".to_owned(),
                value: 0,
            })),
            tmp_list.get("test-name")
        );
    }
    #[test]
    fn set_to_update() {
        let tmp_list: ConfigList<MockConfigItem> = ConfigList::new();

        tmp_list.set(MockConfigItem {
            name: "test-name".to_owned(),
            value: 0,
        });

        let raw_first_get = tmp_list.get("test-name");
        assert_eq!(true, raw_first_get.is_some());
        let first_get = raw_first_get.unwrap();

        tmp_list.set(MockConfigItem {
            name: "test-name".to_owned(),
            value: 2,
        });

        let raw_second_get = tmp_list.get("test-name");
        assert_eq!(true, raw_second_get.is_some());
        let second_get = raw_second_get.unwrap();
        assert_eq!(first_get.get(), second_get.get());
    }
    #[test]
    fn set_get_entry_invalid() {
        let tmp_list: ConfigList<MockConfigItem> = ConfigList::new();

        tmp_list.set(MockConfigItem {
            name: "test-name".to_owned(),
            value: 0,
        });

        assert_eq!(None, tmp_list.get("other-name"));
    }

    #[test]
    fn set_remove_entry_valid() {
        let tmp_list: ConfigList<MockConfigItem> = ConfigList::new();

        tmp_list.set(MockConfigItem {
            name: "test-name".to_owned(),
            value: 0,
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
            value: 0,
        });

        tmp_list.remove("other-name");

        let mut result_map: std::collections::BTreeMap<String, Shared<MockConfigItem>> =
            std::collections::BTreeMap::new();
        result_map.insert(
            "test-name".to_owned(),
            Shared::new(MockConfigItem {
                name: "test-name".to_owned(),
                value: 0,
            }),
        );
        assert_eq!(result_map, tmp_list.entries.lock().unwrap().clone());
    }

    #[test]
    fn get_with_default_already_exists() {
        let tmp_list: ConfigList<MockConfigItem> = ConfigList::new();

        tmp_list.set(MockConfigItem {
            name: "test-config".to_owned(),
            value: 132,
        });

        assert_eq!(
            Shared::new(MockConfigItem {
                name: "test-config".to_owned(),
                value: 132,
            }),
            tmp_list.get_with_default("test-config")
        );
    }
    #[test]
    fn get_with_default_doesnt_exist() {
        let tmp_list: ConfigList<MockConfigItem> = ConfigList::new();

        tmp_list.set(MockConfigItem {
            name: "test-config".to_owned(),
            value: 132,
        });

        assert_eq!(
            Shared::new(MockConfigItem::default_name("other-config".to_owned())),
            tmp_list.get_with_default("other-config")
        );
    }
}
