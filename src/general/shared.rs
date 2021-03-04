#[derive(Debug)]
pub struct Shared<T> {
    /// This holds the actual current Data
    data: std::sync::Arc<std::sync::atomic::AtomicPtr<std::sync::Arc<T>>>,

    /// This holds the previous version of the Data
    versions: std::sync::Arc<std::sync::Mutex<Vec<std::sync::Arc<T>>>>,
}

impl<T> PartialEq for Shared<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let own_data = self.get();
        let other_data = other.get();

        own_data.eq(&other_data)
    }
}

impl<T> Shared<T> {
    pub fn new(inital_value: T) -> Self {
        let arc = std::sync::Arc::new(inital_value);
        let boxed = Box::new(arc.clone());
        let ptr = Box::into_raw(boxed);

        let atom_ptr = std::sync::Arc::new(std::sync::atomic::AtomicPtr::new(ptr));

        Self {
            data: atom_ptr,
            versions: std::sync::Arc::new(std::sync::Mutex::new(vec![arc])),
        }
    }

    /// Replaces the Data with the New-Value, all
    /// following reads will then use this new
    /// value going forward. However the data returned
    /// by reads before this is not changed.
    pub fn update(&self, n_value: T) {
        let n_arc = std::sync::Arc::new(n_value);

        // Clean up the old value
        let mut inner_versions = self.versions.lock().unwrap();
        while inner_versions.len() > 1 {
            inner_versions.pop();
        }
        inner_versions.insert(0, n_arc.clone());

        // Replace the current one with the new one

        let n_boxed = Box::new(n_arc);
        let n_ptr = Box::into_raw(n_boxed);
        let prev_ptr = self.data.swap(n_ptr, std::sync::atomic::Ordering::Relaxed);

        // Clean up the previously allocated memory
        let prev_boxed = unsafe { Box::from_raw(prev_ptr) };
        drop(prev_boxed);
    }

    /// Returns the current Value
    pub fn get(&self) -> std::sync::Arc<T> {
        let raw_ptr = self.data.load(std::sync::atomic::Ordering::Relaxed);

        let boxed = unsafe { Box::from_raw(raw_ptr) };

        let n_arc = std::sync::Arc::clone(&boxed);
        Box::leak(boxed);

        n_arc
    }
}

impl<T> Clone for Shared<T> {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
            versions: self.versions.clone(),
        }
    }
}

#[test]
fn new_get() {
    let tmp_shared = Shared::new(10u8);
    let first_arc = tmp_shared.get();
    assert_eq!(std::sync::Arc::new(10u8), first_arc);
}

#[test]
fn new_get_update_get() {
    let tmp_shared = Shared::new(10u8);
    let first_arc = tmp_shared.get();
    assert_eq!(std::sync::Arc::new(10u8), first_arc);

    tmp_shared.update(15);

    let second_arc = tmp_shared.get();
    assert_eq!(std::sync::Arc::new(15u8), second_arc);
}

#[test]
fn cloned_update() {
    let tmp_shared = Shared::new(10u8);
    let first_arc = tmp_shared.get();
    assert_eq!(std::sync::Arc::new(10u8), first_arc);

    let second_shared = tmp_shared.clone();
    let second_arc = second_shared.get();
    assert_eq!(std::sync::Arc::new(10u8), second_arc);

    tmp_shared.update(15u8);

    assert_eq!(std::sync::Arc::new(15u8), tmp_shared.get());
    assert_eq!(std::sync::Arc::new(15u8), second_shared.get());
}

#[test]
fn is_dropped() {
    let tmp_shared = Shared::new(10u8);
    let first_arc = tmp_shared.get();

    tmp_shared.update(11);
    tmp_shared.update(12);
    tmp_shared.update(13);

    assert_eq!(1, std::sync::Arc::strong_count(&first_arc));
    assert_eq!(std::sync::Arc::new(13u8), tmp_shared.get());
}
