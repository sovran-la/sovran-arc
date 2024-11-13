# SOVRAN-ARC

This library provides convenient wrapper types that combine `Arc` and `Mutex` for safe shared mutable access across threads, drawing inspiration from Swift's reference counting and memory management patterns.

## Types

### Arcm<T> - Arc-Mutex Wrapper
A thread-safe wrapper combining `Arc` and `Mutex` for shared mutable access to a value:
```rust
let counter = Arcm::new(0);
counter.modify(|n| *n += 1);
assert_eq!(counter.value(), 1);
```

Key features:
- Thread-safe shared mutable access
- Clean API for modifications and value access
- Automatic cloning of the internal value
- Default implementation for types implementing `Default`
- Implements `Debug`, `Clone`, and `From`

### WeakArcm<T> - Weak Reference Companion
A weak reference version of `Arcm` that doesn't prevent deallocation:
```rust
let strong = Arcm::new(42);
let weak = strong.downgrade();

// Access if still alive
if let Some(value) = weak.value() {
    println!("Value still exists: {}", value);
}
```

### Arcmo<T> - Optional Arc-Mutex Wrapper
Similar to `Arcm` but wraps an `Option<T>`, providing nullable semantics:
```rust
let value = Arcmo::some(42);
assert_eq!(value.value(), Some(42));

value.take();  // Remove the value
assert!(value.is_none());

value.replace(100);  // Set a new value
assert_eq!(value.value(), Some(100));
```

Key features:
- All the benefits of `Arcm`
- Optional value semantics
- Methods like `take()` and `replace()`
- `is_some()` and `is_none()` checks
- Default implementation creates an empty (None) instance

### WeakArcmo<T> - Optional Weak Reference
Weak reference version of `Arcmo`:
```rust
let strong = Arcmo::some(42);
let weak = strong.downgrade();

assert!(weak.is_some());
assert_eq!(weak.value(), Some(42));
```

## Swift-like Characteristics

This library brings several Swift-like memory management features to Rust:

1. **Reference Counting**: Like Swift's `strong` and `weak` references, these types provide explicit reference counting with strong (`Arcm`/`Arcmo`) and weak (`WeakArcm`/`WeakArcmo`) variants.

2. **Safe Shared Mutability**: Similar to Swift's class instances, these types allow safe shared mutable access across multiple references.

3. **Optional Value Semantics**: `Arcmo` provides similar semantics to Swift's optional references, allowing for nullable shared references.

4. **Clean API**: The API design focuses on ergonomics and safety, similar to Swift's emphasis on safe and expressive APIs.

## Thread Safety

All types in this library are thread-safe and can be safely shared across threads:

- Internal mutability is handled through `Mutex`
- Reference counting is atomic through `Arc`
- Safe to clone and share across thread boundaries
- Deadlock protection through scoped locks

## Usage Examples

### Basic Usage
```rust
// Create a shared counter
let counter = Arcm::new(0);

// Clone and share across threads
let counter2 = counter.clone();
std::thread::spawn(move || {
    counter2.modify(|n| *n += 1);
});

// Modify in main thread
counter.modify(|n| *n += 1);
```

### Optional Values
```rust
// Create an optional shared value
let value = Arcmo::some("Hello");

// Share with another thread
let value2 = value.clone();
std::thread::spawn(move || {
    if value2.is_some() {
        value2.modify(|s| *s = "World");
    }
});

// Take the value if it exists
if let Some(v) = value.take() {
    println!("Taken value: {}", v);
}
```

### Weak References
```rust
let strong = Arcm::new(vec![1, 2, 3]);
let weak = strong.downgrade();

// Modify through weak reference
weak.modify(|v| v.push(4));

// Check if value still exists
if let Some(vec) = weak.value() {
    println!("Vector: {:?}", vec);
}
```

## Requirements

- Rust 1.56 or later
- Types must implement `Clone`
- Optional: `Debug` for debug formatting
- Optional: `Default` for default implementation

## License

Copyright 2024 Sovran.la, Inc.

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the “Software”), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
