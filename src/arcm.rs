use std::fmt::Debug;
use std::sync::{Arc, Mutex, Weak};

/// A wrapper combining Arc and Mutex for convenient shared mutable access
/// Only works with types that implement Clone
pub struct Arcm<T: Clone> {
    inner: Arc<Mutex<T>>,
}

impl<T: Clone> Arcm<T> {
    /// Creates a new Arcm containing the given value
    pub fn new(value: T) -> Self {
        Self {
            inner: Arc::new(Mutex::new(value)),
        }
    }

    /// Modifies the contained value using the provided closure
    pub fn modify<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        let mut guard = self
            .inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        f(&mut *guard)
    }

    /// Returns a copy of the contained value
    pub fn value(&self) -> T {
        match self.inner.lock() {
            Ok(guard) => guard.clone(),
            Err(poisoned) => poisoned.into_inner().clone(),
        }
    }

    /// Returns a weak reference to the contained value
    pub fn downgrade(&self) -> WeakArcm<T> {
        WeakArcm {
            inner: Arc::downgrade(&self.inner),
        }
    }

    /// Replace the value without cloning the old one, returns the old value.
    pub fn replace(&self, value: T) -> T {
        let mut guard = match self.inner.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        std::mem::replace(&mut *guard, value)
    }
}

impl<T: Clone> Clone for Arcm<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<T: Clone + Debug> Debug for Arcm<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Arcm").field("inner", &self.inner).finish()
    }
}

impl<T: Clone + Default> Default for Arcm<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T: Clone> From<T> for Arcm<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

/// A weak reference wrapper for Arcm
pub struct WeakArcm<T: Clone> {
    inner: Weak<Mutex<T>>,
}

impl<T: Clone> WeakArcm<T> {
    /// Attempts to modify the value if the original Arcm still exists
    pub fn modify<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&mut T) -> R,
    {
        self.inner.upgrade().map(|arc| {
            let mut guard = arc.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
            f(&mut *guard)
        })
    }

    /// Attempts to get a copy of the value if the original Arcm still exists
    pub fn value(&self) -> Option<T> {
        self.inner.upgrade().map(|arc| match arc.lock() {
            Ok(guard) => guard.clone(),
            Err(poisoned) => poisoned.into_inner().clone(),
        })
    }

    /// Attempts to replace the value if the original Arcm still exists
    pub fn replace(&self, value: T) -> Option<T> {
        self.inner.upgrade().map(|arc| {
            let mut guard = arc.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
            std::mem::replace(&mut *guard, value)
        })
    }
}

impl<T: Clone> Debug for WeakArcm<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WeakArcm")
            .field("inner", &self.inner)
            .finish()
    }
}

// Example usage and tests
#[cfg(test)]
mod tests {
    use super::*;
    use std::panic::{self, AssertUnwindSafe};
    use std::thread;

    #[test]
    fn test_basic_usage() {
        let v = Arcm::new(1);

        v.modify(|v| *v = 42);
        assert_eq!(v.value(), 42);
    }

    #[test]
    fn test_multiple_references() {
        let v1 = Arcm::new(1);
        let v2 = v1.clone();

        v1.modify(|v| *v = 42);
        assert_eq!(v2.value(), 42);
    }

    #[test]
    fn test_complex_modification() {
        let numbers = Arcm::new(Vec::new());

        numbers.modify(|v| v.push(1));
        numbers.modify(|v| v.push(2));

        assert_eq!(numbers.value(), vec![1, 2]);
    }

    #[test]
    fn test_weak_reference() {
        let strong = Arcm::new(42);
        let weak = strong.downgrade();

        // Much cleaner access
        assert_eq!(weak.value(), Some(42));

        // After dropping the strong reference
        drop(strong);
        assert_eq!(weak.value(), None);
    }

    #[test]
    fn test_weak_modification() {
        let strong = Arcm::new(vec![1, 2, 3]);
        let weak = strong.downgrade();

        // Modify through weak reference
        let length = weak.modify(|v| {
            v.push(4);
            v.len()
        });
        assert_eq!(length, Some(4));
        assert_eq!(strong.value(), vec![1, 2, 3, 4]);

        // After dropping the strong reference
        drop(strong);
        let result = weak.modify(|v| v.push(5));
        assert_eq!(result, None);
    }

    #[test]
    fn test_default() {
        // Creates an Arcm containing an empty Vec
        let vec_arcm: Arcm<Vec<i32>> = Arcm::default();
        assert_eq!(vec_arcm.value(), Vec::new());

        // Creates an Arcm containing 0
        let int_arcm: Arcm<i32> = Arcm::default();
        assert_eq!(int_arcm.value(), 0);

        // Creates an Arcm containing empty String
        let string_arcm: Arcm<String> = Arcm::default();
        assert_eq!(string_arcm.value(), String::new());
    }

    #[test]
    fn test_from() {
        // Using From directly
        let arcm1 = Arcm::from(42);
        assert_eq!(arcm1.value(), 42);

        // Using Into (which is automatically implemented when From is implemented)
        let arcm2: Arcm<String> = "hello".to_string().into();
        assert_eq!(arcm2.value(), "hello");

        // Using into() method - with explicit type annotation
        let arcm3: Arcm<Vec<i32>> = Vec::new().into();
        assert!(arcm3.value().is_empty());
    }

    #[test]
    fn test_arcm_poisoned_mutex_recovery() {
        let arcm = Arcm::new(42);
        let arcm_clone = arcm.clone();

        // Poison the mutex by causing a panic while holding the lock
        let _ = panic::catch_unwind(AssertUnwindSafe(|| {
            let handle = thread::spawn(move || {
                // This will poison the mutex
                arcm_clone.modify(|_| panic!("Deliberate panic to poison mutex"));
            });

            // Wait for the thread to complete (and panic)
            let _ = handle.join();
        }));

        // Now try to use the poisoned mutex - should recover
        let value = arcm.value();
        assert_eq!(value, 42);

        // Try to modify the poisoned mutex - should recover
        let result = arcm.modify(|v| {
            *v = 100;
            *v
        });
        assert_eq!(result, 100);
        assert_eq!(arcm.value(), 100);
    }

    #[test]
    fn test_arcm_replace() {
        let arcm = Arcm::new(42);

        // Test basic replace functionality
        let old_value = arcm.replace(100);
        assert_eq!(old_value, 42);
        assert_eq!(arcm.value(), 100);

        // Test replace with multiple references
        let arcm2 = arcm.clone();
        arcm.replace(200);
        assert_eq!(arcm2.value(), 200);

        // Test replace with complex types
        let vec_arcm = Arcm::new(vec![1, 2, 3]);
        let old_vec = vec_arcm.replace(vec![4, 5, 6]);
        assert_eq!(old_vec, vec![1, 2, 3]);
        assert_eq!(vec_arcm.value(), vec![4, 5, 6]);
    }

    #[test]
    fn test_arcm_replace_with_poisoned_mutex() {
        let arcm = Arcm::new(42);
        let arcm_clone = arcm.clone();

        // Poison the mutex
        let _ = panic::catch_unwind(AssertUnwindSafe(|| {
            let handle = thread::spawn(move || {
                arcm_clone.modify(|_| panic!("Deliberate panic to poison mutex"));
            });
            let _ = handle.join();
        }));

        // Try replace with poisoned mutex - should recover
        let old_value = arcm.replace(100);
        assert_eq!(old_value, 42);
        assert_eq!(arcm.value(), 100);
    }

    #[test]
    fn test_weak_arcm_poisoned_mutex() {
        let strong = Arcm::new(42);
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

        let result = weak.modify(|v| {
            *v = 100;
            *v
        });
        assert_eq!(result, Some(100));
        assert_eq!(strong.value(), 100);
    }

    #[test]
    fn test_weak_arcm_replace() {
        let strong = Arcm::new(42);
        let weak = strong.downgrade();

        // Test basic replace
        let old_value = weak.replace(100);
        assert_eq!(old_value, Some(42));
        assert_eq!(strong.value(), 100);

        // Test replace after dropping strong reference
        drop(strong);
        let result = weak.replace(200);
        assert_eq!(result, None); // Should return None when strong ref is gone
    }

    #[test]
    fn test_weak_arcm_poisoned_and_replace() {
        let strong = Arcm::new(42);
        let weak = strong.downgrade();
        let strong_clone = strong.clone();

        // Poison the mutex
        let _ = panic::catch_unwind(AssertUnwindSafe(|| {
            let handle = thread::spawn(move || {
                strong_clone.modify(|_| panic!("Deliberate panic to poison mutex"));
            });
            let _ = handle.join();
        }));

        // Test replace with poisoned mutex
        let old_value = weak.replace(100);
        assert_eq!(old_value, Some(42));
        assert_eq!(strong.value(), 100);
    }

    #[test]
    fn test_arcm_thread_safety() {
        let arcm = Arcm::new(0);
        let threads = 10;
        let increments_per_thread = 1000;

        let handles: Vec<_> = (0..threads)
            .map(|_| {
                let arcm = arcm.clone();
                thread::spawn(move || {
                    for _ in 0..increments_per_thread {
                        arcm.modify(|v| *v += 1);
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(arcm.value(), threads * increments_per_thread);
    }
}
