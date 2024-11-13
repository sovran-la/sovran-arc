use sovran_arc::arcmo::Arcmo;

fn main() {
    println!("=== Creating Arcmo instances ===");
    // Create with Some value
    let v = Arcmo::some(42);
    println!("Created with some: v = {:?}", v);

    // Create with Default (None)
    let default: Arcmo<i32> = Arcmo::default();
    println!("Created with default: default = {:?}", default);

    // Create explicitly as None
    let none: Arcmo<i32> = Arcmo::none();
    println!("Created as none: none = {:?}", none);

    println!("\n=== Basic value operations ===");
    println!("v.value() = {:?}", v.value());
    println!("v.is_some() = {}", v.is_some());
    println!("v.is_none() = {}", v.is_none());

    println!("\n=== Modification ===");
    // Modify existing value
    v.modify(|x| *x += 1);
    println!("After v += 1: {:?}", v.value());

    v.modify(|x| *x *= 2);
    println!("After v *= 2: {:?}", v.value());

    // Try to modify None (should have no effect)
    none.modify(|x| *x = 100);
    println!("After trying to modify None: {:?}", none.value());

    println!("\n=== Multiple references ===");
    let v2 = v.clone();
    v.modify(|x| *x += 5);
    println!("Modified through v: {:?}", v.value());
    println!("Observed through v2: {:?}", v2.value());

    println!("\n=== Weak references ===");
    let weak = v.downgrade();
    println!("Weak reference: {:?}", weak);
    println!("weak.value() = {:?}", weak.value());
    println!("weak.is_some() = {}", weak.is_some());
    println!("weak.is_none() = {}", weak.is_none());

    // Modify through weak reference
    weak.modify(|x| *x += 10);
    println!("\nAfter weak modify:");
    println!("Observed through strong ref: {:?}", v.value());
    println!("Observed through weak ref: {:?}", weak.value());

    println!("\n=== Take and Replace operations ===");
    println!("Taking value from v: {:?}", v.take());
    println!("After take:");
    println!("v.is_none() = {}", v.is_none());
    println!("weak.is_none() = {}", weak.is_none());

    println!("\nReplacing with new value:");
    let previous = v.replace(100);
    println!("Previous value: {:?}", previous);
    println!("New value through v: {:?}", v.value());
    println!("New value through weak: {:?}", weak.value());

    println!("\n=== Weak reference behavior when strong ref is dropped ===");
    {
        let temp = Arcmo::some(999);
        let weak_temp = temp.downgrade();
        println!("Before drop - weak_temp.value() = {:?}", weak_temp.value());
        drop(temp);
        println!("After drop - weak_temp.value() = {:?}", weak_temp.value());
    }

    println!("\n=== Complex type example ===");
    let vec_arcmo = Arcmo::some(vec![1, 2, 3]);
    println!("Vector Arcmo: {:?}", vec_arcmo);

    vec_arcmo.modify(|v| v.push(4));
    println!("After push: {:?}", vec_arcmo.value());

    let weak_vec = vec_arcmo.downgrade();
    weak_vec.modify(|v| v.extend_from_slice(&[5, 6]));
    println!("After weak modify: {:?}", vec_arcmo.value());

    println!("\n=== Default for different types ===");
    let default_vec: Arcmo<Vec<i32>> = Arcmo::default();
    let default_string: Arcmo<String> = Arcmo::default();
    println!("Default Vec<i32>: {:?}", default_vec);
    println!("Default String: {:?}", default_string);
}