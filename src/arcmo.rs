use std::fmt::Debug;
use std::sync::{Arc, Mutex, Weak};

/// A wrapper combining Arc and Mutex for convenient shared mutable access to optional values
/// Only works with types that implement Clone
pub struct Arcmo<T: Clone> {
    inner: Arc<Mutex<Option<T>>>,
}

impl<T: Clone> Arcmo<T> {
    /// Creates a new empty Arcmo
    pub fn none() -> Self {
        Self {
            inner: Arc::new(Mutex::new(None)),
        }
    }

    /// Creates a new Arcmo containing Some(value)
    pub fn some(value: T) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Some(value))),
        }
    }

    /// Modifies the contained value using the provided closure.
    /// If no value exists, creates one using T::Default before applying the modification.
    /// Returns the result of the closure.
    pub fn modify<F, R>(&self, f: F) -> R
    where
        T: Default,
        F: FnOnce(&mut T) -> R,
    {
        let mut guard = self
            .inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        match &mut *guard {
            Some(value) => f(value),
            None => {
                let mut value = T::default();
                let result = f(&mut value);
                *guard = Some(value);
                result
            }
        }
    }

    /// Sets the value to None and returns the previous value if it existed
    pub fn take(&self) -> Option<T> {
        let mut guard = self
            .inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        guard.take()
    }

    /// Sets the value to Some(value) and returns the previous value if it existed
    pub fn replace(&self, value: T) -> Option<T> {
        let mut guard = self
            .inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        guard.replace(value)
    }

    /// Returns a copy of the contained value if it exists
    pub fn value(&self) -> Option<T> {
        let guard = self
            .inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        guard.clone()
    }

    /// Returns true if the contained value is Some
    pub fn is_some(&self) -> bool {
        let guard = self
            .inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        guard.is_some()
    }

    /// Returns true if the contained value is None
    pub fn is_none(&self) -> bool {
        let guard = self
            .inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        guard.is_none()
    }

    /// Returns a weak reference to the contained value
    pub fn downgrade(&self) -> WeakArcmo<T> {
        WeakArcmo {
            inner: Arc::downgrade(&self.inner),
        }
    }
}

impl<T: Clone> Clone for Arcmo<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<T: Clone + Debug> Debug for Arcmo<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Arcmo").field("inner", &self.inner).finish()
    }
}

impl<T: Clone + Default> Default for Arcmo<T> {
    fn default() -> Self {
        Self::none()
    }
}

/// A weak reference wrapper for Arcmo
pub struct WeakArcmo<T: Clone> {
    inner: Weak<Mutex<Option<T>>>,
}

impl<T: Clone> WeakArcmo<T> {
    /// Attempts to modify the value if it exists and the original Arcmo still exists
    pub fn modify<F, R>(&self, f: F) -> Option<R>
    where
        T: Default,
        F: FnOnce(&mut T) -> R,
    {
        self.inner.upgrade().map(|arc| {
            let mut guard = arc.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
            match &mut *guard {
                Some(value) => f(value),
                None => {
                    let mut value = T::default();
                    let result = f(&mut value);
                    *guard = Some(value);
                    result
                }
            }
        })
    }

    /// Attempts to get a copy of the value if it exists and the original Arcmo still exists
    pub fn value(&self) -> Option<T> {
        self.inner.upgrade().and_then(|arc| match arc.lock() {
            Ok(guard) => guard.clone(),
            Err(poisoned) => poisoned.into_inner().clone(),
        })
    }

    /// Returns true if both the original Arcmo exists and contains Some value
    pub fn is_some(&self) -> bool {
        self.inner
            .upgrade()
            .map(|arc| match arc.lock() {
                Ok(guard) => guard.is_some(),
                Err(poisoned) => poisoned.into_inner().is_some(),
            })
            .unwrap_or(false)
    }

    /// Returns true if either the original Arcmo is dropped or contains None
    pub fn is_none(&self) -> bool {
        !self.is_some()
    }

    /// Attempts to replace the value if the original Arcmo still exists
    pub fn replace(&self, value: T) -> Option<Option<T>> {
        self.inner.upgrade().map(|arc| {
            let mut guard = arc.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
            std::mem::replace(&mut *guard, Some(value))
        })
    }
}

impl<T: Clone> Debug for WeakArcmo<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WeakArcmo")
            .field("inner", &self.inner)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::panic::{self, AssertUnwindSafe};
    use std::thread;

    #[derive(Clone, Debug, Default, PartialEq)]
    struct Settings {
        enabled: bool,
        count: i32,
        name: String,
    }

    impl Settings {
        fn update_timestamp(&mut self) {
            self.count += 1;
        }

        fn recalculate_dependencies(&mut self) {
            if self.enabled {
                self.count *= 2;
            }
        }
    }

    #[test]
    fn test_default() {
        let arcmo: Arcmo<Vec<i32>> = Arcmo::default();
        assert!(arcmo.is_none());

        let int_arcmo: Arcmo<i32> = Arcmo::default();
        assert!(int_arcmo.is_none());
    }

    #[test]
    fn test_basic_usage() {
        let v = Arcmo::some(1);
        let result = v.modify(|v| {
            *v = 42;
            *v
        });
        assert_eq!(result, 42);
        assert_eq!(v.value(), Some(42));
    }

    #[test]
    fn test_modification_patterns() {
        // Set whole struct
        let state = Arcmo::none();
        let new_state = Settings {
            enabled: true,
            count: 42,
            name: "test".to_string(),
        };
        let result = state.modify(|s: &mut Settings| {
            *s = new_state.clone();
            s.count
        });
        assert_eq!(result, 42);
        assert_eq!(state.value(), Some(new_state));

        // Update one field
        let counter = Arcmo::<Settings>::none();
        let result = counter.modify(|s: &mut Settings| {
            s.count += 1;
            s.count
        });
        assert_eq!(result, 1);
        assert_eq!(counter.value().unwrap().count, 1);

        // Call methods
        let vec = Arcmo::<Vec<i32>>::none();
        let len = vec.modify(|v: &mut Vec<i32>| {
            v.push(42);
            v.len()
        });
        assert_eq!(len, 1);
        assert_eq!(vec.value(), Some(vec![42]));

        // Complex updates
        let settings = Arcmo::<Settings>::none();
        let count = settings.modify(|s: &mut Settings| {
            s.enabled = true;
            s.update_timestamp();
            s.recalculate_dependencies();
            s.count
        });
        assert_eq!(count, 2); // 1 from timestamp, then doubled
        let result = settings.value().unwrap();
        assert!(result.enabled);
        assert_eq!(result.count, 2);
    }

    #[test]
    fn test_take_and_replace() {
        let v = Arcmo::some(1);
        assert_eq!(v.take(), Some(1));
        assert!(v.is_none());
        assert_eq!(v.replace(42), None);
        assert_eq!(v.value(), Some(42));
    }

    #[test]
    fn test_multiple_references() {
        let v1 = Arcmo::some(1);
        let v2 = v1.clone();

        let result = v1.modify(|v| {
            *v = 42;
            *v
        });
        assert_eq!(result, 42);
        assert_eq!(v2.value(), Some(42));

        v1.take();
        assert!(v2.is_none());
    }

    #[test]
    fn test_modify_none_uses_default() {
        let settings: Arcmo<Settings> = Arcmo::none();

        let count = settings.modify(|s: &mut Settings| {
            assert!(!s.enabled);
            assert_eq!(s.count, 0);
            assert_eq!(s.name, "");
            s.enabled = true;
            s.count
        });

        assert_eq!(count, 0);
        let result = settings.value().unwrap();
        assert!(result.enabled);
        assert_eq!(result.count, 0);
        assert_eq!(result.name, "");
    }

    #[test]
    fn test_weak_reference() {
        let strong = Arcmo::some(42);
        let weak = strong.downgrade();
        assert_eq!(weak.value(), Some(42));
        drop(strong);
        assert_eq!(weak.value(), None);
    }

    #[test]
    fn test_weak_with_none() {
        let strong = Arcmo::none();
        let weak = strong.downgrade();

        assert_eq!(weak.value(), None);
        assert!(weak.is_none());
        assert!(!weak.is_some());

        strong.replace(42);
        assert_eq!(weak.value(), Some(42));
        assert!(!weak.is_none());
        assert!(weak.is_some());

        strong.take();
        assert_eq!(weak.value(), None);
        assert!(weak.is_none());
        assert!(!weak.is_some());
    }

    #[test]
    fn test_weak_modification() {
        let strong = Arcmo::some(vec![1, 2, 3]);
        let weak = strong.downgrade();

        // Modify through weak reference
        let length = weak.modify(|v| {
            v.push(4);
            v.len()
        });
        assert_eq!(length, Some(4));
        assert_eq!(strong.value(), Some(vec![1, 2, 3, 4]));

        // Test with None -> Default
        let strong = Arcmo::<Vec<i32>>::none();
        let weak = strong.downgrade();
        let len = weak.modify(|v| {
            v.push(42);
            v.len()
        });
        assert_eq!(len, Some(1));
        assert_eq!(strong.value(), Some(vec![42]));

        // After dropping the strong reference
        drop(strong);
        let result = weak.modify(|v| {
            v.push(5);
            v.len()
        });
        assert_eq!(result, None);
    }

    #[test]
    fn test_arcmo_poisoned_mutex_recovery() {
        let arcmo = Arcmo::some(42);
        let arcmo_clone = arcmo.clone();

        // Poison the mutex by causing a panic while holding the lock
        let _ = panic::catch_unwind(AssertUnwindSafe(|| {
            let handle = thread::spawn(move || {
                // This will poison the mutex
                arcmo_clone.modify(|_| panic!("Deliberate panic to poison mutex"));
            });

            // Wait for the thread to complete (and panic)
            let _ = handle.join();
        }));

        // Now try to use the poisoned mutex - should recover
        let value = arcmo.value();
        assert_eq!(value, Some(42));

        // Try to modify the poisoned mutex - should recover
        arcmo.modify(|v| *v = 100);
        assert_eq!(arcmo.value(), Some(100));

        // Test other methods with poisoned mutex
        assert!(arcmo.is_some());
        assert!(!arcmo.is_none());

        let taken = arcmo.take();
        assert_eq!(taken, Some(100));
        assert!(arcmo.is_none());

        let replaced = arcmo.replace(200);
        assert_eq!(replaced, None);
        assert_eq!(arcmo.value(), Some(200));
    }

    #[test]
    fn test_weak_arcmo_replace() {
        // Test with Some value
        let strong = Arcmo::some(42);
        let weak = strong.downgrade();

        let prev_value = weak.replace(100);
        assert_eq!(prev_value, Some(Some(42))); // Previous value was Some(42)
        assert_eq!(strong.value(), Some(100)); // New value is 100

        // Test with None value
        let strong_none = Arcmo::<i32>::none();
        let weak_none = strong_none.downgrade();

        let prev_value = weak_none.replace(200);
        assert_eq!(prev_value, Some(None)); // Previous value was None
        assert_eq!(strong_none.value(), Some(200)); // New value is 200

        // Test with dropped strong reference
        drop(strong_none);
        let result = weak_none.replace(300);
        assert_eq!(result, None); // Should return None when strong ref is gone
    }

    #[test]
    fn test_weak_arcmo_poisoned_mutex() {
        let strong = Arcmo::some(42);
        let weak = strong.downgrade();
        let strong_clone = strong.clone();

        // Poison the mutex
        let _ = panic::catch_unwind(AssertUnwindSafe(|| {
            let handle = thread::spawn(move || {
                strong_clone.modify(|_| panic!("Deliberate panic to poison mutex"));
            });
            let _ = handle.join();
        }));

        // Try weak methods with poisoned mutex
        let value = weak.value();
        assert_eq!(value, Some(42));

        assert!(weak.is_some());
        assert!(!weak.is_none());

        let result = weak.modify(|v| {
            *v = 100;
            *v
        });
        assert_eq!(result, Some(100));
        assert_eq!(strong.value(), Some(100));

        // Test replace with poisoned mutex
        let old_value = weak.replace(200);
        assert_eq!(old_value, Some(Some(100)));
        assert_eq!(strong.value(), Some(200));
    }

    #[test]
    fn test_weak_arcmo_none_to_some() {
        // Test upgrading None to Some via replace
        let strong = Arcmo::<i32>::none();
        let weak = strong.downgrade();

        assert!(strong.is_none());
        assert!(weak.is_none());

        // Replace None with Some
        let prev = weak.replace(42);
        assert_eq!(prev, Some(None));
        assert!(strong.is_some());
        assert!(weak.is_some());
        assert_eq!(strong.value(), Some(42));
    }
}
