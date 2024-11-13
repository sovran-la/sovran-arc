use sovran_arc::arcm::Arcm;

fn main() {
    let v = Arcm::new(42);
    println!("v = {:?}", v);
    println!("v.value() = {}", v.value());

    v.modify(|x| *x += 1);
    println!("v = {:?}", v);
    println!("v.value() = {}", v.value());

    v.modify(|x| *x *= 2);
    println!("v = {:?}", v);
    println!("v.value() = {}", v.value());

    // Create a weak reference
    let weak = v.downgrade();
    println!("weak = {:?}", weak);
    println!("weak.value() = {:?}", weak.value());

    // Modify through weak reference
    weak.modify(|x| *x += 10);
    println!("After weak modify, v.value() = {}", v.value());

    // Demonstrate what happens when strong reference is dropped
    drop(v);
    println!("After dropping strong ref, weak.value() = {:?}", weak.value());

    // Examples of From and Into
    println!("\n=== From and Into Examples ===");

    // Using From
    let v2 = Arcm::from(100);
    println!("\nUsing From:");
    println!("v2 = {:?}", v2);
    println!("v2.value() = {}", v2.value());

    // Using Into
    let v3: Arcm<i32> = 200.into();
    println!("\nUsing Into:");
    println!("v3 = {:?}", v3);
    println!("v3.value() = {}", v3.value());

    // Using Into with String
    let str_arcm: Arcm<String> = "Hello, World!".to_string().into();
    println!("\nUsing Into with String:");
    println!("str_arcm = {:?}", str_arcm);
    println!("str_arcm.value() = {}", str_arcm.value());

    // Using Into with Vec
    let vec_arcm: Arcm<Vec<i32>> = vec![1, 2, 3].into();
    println!("\nUsing Into with Vec:");
    println!("vec_arcm = {:?}", vec_arcm);
    println!("vec_arcm.value() = {:?}", vec_arcm.value());
}