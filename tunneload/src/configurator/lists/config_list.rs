use std::{fmt::Debug, sync::Arc};

use general::{Name, Shared};
use general_traits::{ConfigItem, DefaultConfig};

/// A List of different Types of Configurations
#[derive(Debug)]
pub struct ConfigList<C>
where
    C: ConfigItem + Debug,
{
    entries: std::sync::Arc<std::sync::Mutex<std::collections::HashMap<Name, Shared<C>>>>,
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
    /// Creates a new empty List
    pub fn new() -> Self {
        Self {
            entries: std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
        }
    }

    /// Updates the List to now contain the given Config
    ///
    /// # Behaviour:
    /// If the Config is not in the List, it is inserted into it.
    /// If the Config is in the List, it will be replaced by the new Config
    ///
    /// # Returns:
    /// The new Size of the List
    pub fn set(&self, n_conf: C) -> usize {
        let mut inner = self
            .entries
            .lock()
            .expect("The Lock should always be available");

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

    /// Loads the Config Item that matches the given Name
    pub fn get(&self, name: &Name) -> Option<Shared<C>> {
        let inner = self
            .entries
            .lock()
            .expect("The Lock should always be available");

        inner.get(name).cloned()
    }

    /// Removes the Entry that matches the given Name
    ///
    /// # Returns:
    /// The new Size of the List
    pub fn remove(&self, name: &Name) -> usize {
        let mut inner = self
            .entries
            .lock()
            .expect("The Lock should always be available");

        inner.remove(name);
        inner.len()
    }

    /// Clones the internal List of all ConfigItems currently
    /// registered
    pub fn get_all(&self) -> Vec<Arc<C>> {
        let inner = self
            .entries
            .lock()
            .expect("The Lock should always be available");

        let all_entries = inner.clone();
        drop(inner);

        let mut result = Vec::new();
        for (_, value) in all_entries.iter() {
            result.push(value.get());
        }
        result
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
    pub fn get_with_default(&self, name: Name) -> Shared<C> {
        let mut inner = self
            .entries
            .lock()
            .expect("The Lock should always be available");

        match inner.get(&name) {
            Some(c) => c.clone(),
            None => {
                let n_item = Shared::new(C::default_name(name.clone()));
                inner.insert(name, n_item.clone());
                n_item
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use general::Group;

    use super::*;

    #[derive(Debug, PartialEq)]
    struct MockConfigItem {
        name: Name,
        value: u32,
    }
    impl ConfigItem for MockConfigItem {
        fn name(&self) -> &Name {
            &self.name
        }
    }
    impl DefaultConfig for MockConfigItem {
        fn default_name(name: Name) -> Self {
            Self { name, value: 0 }
        }
    }

    #[test]
    fn set_entry() {
        let tmp_list: ConfigList<MockConfigItem> = ConfigList::new();

        tmp_list.set(MockConfigItem {
            name: Name::new("test-name", Group::Internal),
            value: 0,
        });

        let mut result_map: std::collections::HashMap<Name, Shared<MockConfigItem>> =
            std::collections::HashMap::new();
        result_map.insert(
            Name::new("test-name", Group::Internal),
            Shared::new(MockConfigItem {
                name: Name::new("test-name", Group::Internal),
                value: 0,
            }),
        );
        assert_eq!(result_map, tmp_list.entries.lock().unwrap().clone());
    }

    #[test]
    fn set_get_entry_valid() {
        let tmp_list: ConfigList<MockConfigItem> = ConfigList::new();

        tmp_list.set(MockConfigItem {
            name: Name::new("test-name", Group::Internal),
            value: 0,
        });

        assert_eq!(
            Some(Shared::new(MockConfigItem {
                name: Name::new("test-name", Group::Internal),
                value: 0,
            })),
            tmp_list.get(&Name::new("test-name", Group::Internal))
        );
    }
    #[test]
    fn set_to_update() {
        let tmp_list: ConfigList<MockConfigItem> = ConfigList::new();

        tmp_list.set(MockConfigItem {
            name: Name::new("test-name", Group::Internal),
            value: 0,
        });

        let raw_first_get = tmp_list.get(&Name::new("test-name", Group::Internal));
        assert_eq!(true, raw_first_get.is_some());
        let first_get = raw_first_get.unwrap();

        tmp_list.set(MockConfigItem {
            name: Name::new("test-name", Group::Internal),
            value: 2,
        });

        let raw_second_get = tmp_list.get(&Name::new("test-name", Group::Internal));
        assert_eq!(true, raw_second_get.is_some());
        let second_get = raw_second_get.unwrap();
        assert_eq!(first_get.get(), second_get.get());
    }
    #[test]
    fn set_get_entry_invalid() {
        let tmp_list: ConfigList<MockConfigItem> = ConfigList::new();

        tmp_list.set(MockConfigItem {
            name: Name::new("test-name", Group::Internal),
            value: 0,
        });

        assert_eq!(
            None,
            tmp_list.get(&Name::new("other-name", Group::Internal))
        );
    }

    #[test]
    fn set_remove_entry_valid() {
        let tmp_list: ConfigList<MockConfigItem> = ConfigList::new();

        tmp_list.set(MockConfigItem {
            name: Name::new("test-name", Group::Internal),
            value: 0,
        });

        tmp_list.remove(&Name::new("test-name", Group::Internal));

        assert_eq!(
            std::collections::HashMap::new(),
            tmp_list.entries.lock().unwrap().clone()
        );
    }
    #[test]
    fn set_remove_entry_invalid() {
        let tmp_list: ConfigList<MockConfigItem> = ConfigList::new();

        tmp_list.set(MockConfigItem {
            name: Name::new("test-name", Group::Internal),
            value: 0,
        });

        tmp_list.remove(&Name::new("other-name", Group::Internal));

        let mut result_map: std::collections::HashMap<Name, Shared<MockConfigItem>> =
            std::collections::HashMap::new();
        result_map.insert(
            Name::new("test-name", Group::Internal),
            Shared::new(MockConfigItem {
                name: Name::new("test-name", Group::Internal),
                value: 0,
            }),
        );
        assert_eq!(result_map, tmp_list.entries.lock().unwrap().clone());
    }

    #[test]
    fn get_with_default_already_exists() {
        let tmp_list: ConfigList<MockConfigItem> = ConfigList::new();

        tmp_list.set(MockConfigItem {
            name: Name::new("test-config", Group::Internal),
            value: 132,
        });

        assert_eq!(
            Shared::new(MockConfigItem {
                name: Name::new("test-config", Group::Internal),
                value: 132,
            }),
            tmp_list.get_with_default(Name::new("test-config", Group::Internal))
        );
    }
    #[test]
    fn get_with_default_doesnt_exist() {
        let tmp_list: ConfigList<MockConfigItem> = ConfigList::new();

        tmp_list.set(MockConfigItem {
            name: Name::new("test-config", Group::Internal),
            value: 132,
        });

        assert_eq!(
            Shared::new(MockConfigItem::default_name(Name::new(
                "other-config",
                Group::Internal
            ))),
            tmp_list.get_with_default(Name::new("other-config", Group::Internal))
        );
    }
}
