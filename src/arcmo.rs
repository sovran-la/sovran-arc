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

    /// Modifies the contained value if it exists using the provided closure
    pub fn modify<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&mut T) -> R,
    {
        let mut guard = self.inner.lock().unwrap();
        guard.as_mut().map(f)
    }

    /// Sets the value to None and returns the previous value if it existed
    pub fn take(&self) -> Option<T> {
        self.inner.lock().unwrap().take()
    }

    /// Sets the value to Some(value) and returns the previous value if it existed
    pub fn replace(&self, value: T) -> Option<T> {
        self.inner.lock().unwrap().replace(value)
    }

    /// Returns a copy of the contained value if it exists
    pub fn value(&self) -> Option<T> {
        self.inner.lock().unwrap().clone()
    }

    /// Returns true if the contained value is Some
    pub fn is_some(&self) -> bool {
        self.inner.lock().unwrap().is_some()
    }

    /// Returns true if the contained value is None
    pub fn is_none(&self) -> bool {
        self.inner.lock().unwrap().is_none()
    }

    /// Returns a weak reference to the contained value
    pub fn downgrade(&self) -> WeakArcmo<T> {
        WeakArcmo {
            inner: Arc::downgrade(&self.inner)
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
        f.debug_struct("Arcmo")
            .field("inner", &self.inner)
            .finish()
    }
}

impl<T: Clone + Default> Default for Arcmo<T> {
    fn default() -> Self {
        Self::none()
    }
}

/// A weak reference wrapper for Arcmo
pub struct WeakArcmo<T: Clone> {
    inner: Weak<Mutex<Option<T>>>
}

impl<T: Clone> WeakArcmo<T> {
    /// Attempts to modify the value if it exists and the original Arcmo still exists
    pub fn modify<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&mut T) -> R,
    {
        self.inner
            .upgrade()
            .and_then(|arc| {
                let mut guard = arc.lock().unwrap();
                guard.as_mut().map(f)
            })
    }

    /// Attempts to get a copy of the value if it exists and the original Arcmo still exists
    pub fn value(&self) -> Option<T> {
        self.inner
            .upgrade()
            .and_then(|arc| arc.lock().unwrap().clone())
    }

    /// Returns true if both the original Arcmo exists and contains Some value
    pub fn is_some(&self) -> bool {
        self.inner
            .upgrade()
            .map(|arc| arc.lock().unwrap().is_some())
            .unwrap_or(false)
    }

    /// Returns true if either the original Arcmo is dropped or contains None
    pub fn is_none(&self) -> bool {
        !self.is_some()
    }
}

impl<T: Clone> Debug for WeakArcmo<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WeakArcmo")
            .field("inner", &self.inner)
            .finish()
    }
}

// Example usage and tests
#[cfg(test)]
mod tests {
    use super::*;

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

        v.modify(|v| *v = 42);
        assert_eq!(v.value(), Some(42));
    }

    #[test]
    fn test_none() {
        let v: Arcmo<i32> = Arcmo::none();
        assert!(v.is_none());
        assert_eq!(v.value(), None);

        // modify does nothing when value is None
        v.modify(|v| *v = 42);
        assert_eq!(v.value(), None);

        let v2 = Arcmo::<i32>::none();
        assert!(v2.is_none());
        assert_eq!(v2.value(), None);

        // modify does nothing when value is None
        v.modify(|v2| *v2 = 42);
        assert_eq!(v2.value(), None);
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

        v1.modify(|v| *v = 42);
        assert_eq!(v2.value(), Some(42));

        v1.take();
        assert!(v2.is_none());
    }

    #[test]
    fn test_is_some() {
        // Test with initial Some value
        let v = Arcmo::some(42);
        assert!(v.is_some());

        // Test after modification
        v.modify(|x| *x = 100);
        assert!(v.is_some());

        // Test with None
        let v2: Arcmo<i32> = Arcmo::none();
        assert!(!v2.is_some());

        // Test after taking value
        v.take();
        assert!(!v.is_some());

        // Test after replacing None with Some
        v.replace(200);
        assert!(v.is_some());

        // Test with cloned reference
        let v3 = v.clone();
        assert!(v3.is_some());
    }

    #[test]
    fn test_weak_reference() {
        let strong = Arcmo::some(42);
        let weak = strong.downgrade();

        // Test value access
        assert_eq!(weak.value(), Some(42));

        // Test after dropping the strong reference
        drop(strong);
        assert_eq!(weak.value(), None);
    }

    #[test]
    fn test_weak_with_none() {
        let strong = Arcmo::none();
        let weak = strong.downgrade();

        // Test value access with None
        assert_eq!(weak.value(), None);
        assert!(weak.is_none());
        assert!(!weak.is_some());

        // Replace with Some value
        strong.replace(42);
        assert_eq!(weak.value(), Some(42));
        assert!(!weak.is_none());
        assert!(weak.is_some());

        // Take value back to None
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

        // After dropping the strong reference
        drop(strong);
        let result = weak.modify(|v| v.push(5));
        assert_eq!(result, None);
    }
}