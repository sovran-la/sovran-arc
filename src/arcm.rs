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
        let mut guard = self.inner.lock().unwrap();
        f(&mut *guard)
    }

    /// Returns a copy of the contained value
    pub fn value(&self) -> T {
        self.inner.lock().unwrap().clone()
    }

    /// Returns a weak reference to the contained value
    pub fn downgrade(&self) -> WeakArcm<T> {
        WeakArcm {
            inner: Arc::downgrade(&self.inner)
        }
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
        f.debug_struct("Arcm")
            .field("inner", &self.inner)
            .finish()
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
    inner: Weak<Mutex<T>>
}

impl<T: Clone> WeakArcm<T> {
    /// Attempts to modify the value if the original Arcm still exists
    pub fn modify<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&mut T) -> R,
    {
        self.inner
            .upgrade()
            .map(|arc| {
                let mut guard = arc.lock().unwrap();
                f(&mut *guard)
            })
    }

    /// Attempts to get a copy of the value if the original Arcm still exists
    pub fn value(&self) -> Option<T> {
        self.inner
            .upgrade()
            .map(|arc| arc.lock().unwrap().clone())
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
}