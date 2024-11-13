use std::process::Command;
use std::fs;
use toml_edit::{Document, Item};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read current Cargo.toml
    let cargo_content = fs::read_to_string("Cargo.toml")?;
    let mut doc = cargo_content.parse::<Document>()?;

    // Get current version
    let current_version = doc["package"]["version"]
        .as_str()
        .expect("Could not find version in Cargo.toml");

    // Ask for new version
    println!("Current version is: {}", current_version);
    println!("Enter new version:");
    let mut new_version = String::new();
    std::io::stdin().read_line(&mut new_version)?;
    let new_version = new_version.trim();

    // Update Cargo.toml
    doc["package"]["version"] = Item::from(new_version);
    fs::write("Cargo.toml", doc.to_string())?;

    // Git commands
    let commands = [
        ("git add Cargo.toml", "Failed to stage Cargo.toml"),
        (&format!("git commit -m \"Bump version to {}\"", new_version), "Failed to commit version bump"),
        (&format!("git tag -a v{} -m \"Version {}\"", new_version, new_version), "Failed to create tag"),
        ("git push", "Failed to push commits"),
        ("git push --tags", "Failed to push tags"),
        ("cargo publish", "Failed to publish to crates.io"),
    ];

    for (cmd, error_msg) in commands.iter() {
        let status = Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .status()?;

        if !status.success() {
            return Err(error_msg.to_string().into());
        }
    }

    println!("Successfully released version {}", new_version);
    Ok(())
}