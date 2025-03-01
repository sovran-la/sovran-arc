use std::fs;
use std::io::{self, Write};
use std::process::Command;
use toml_edit::DocumentMut;
use toml_edit::Item;

fn get_commit_history(previous_tag: &str) -> Result<String, Box<dyn std::error::Error>> {
    if previous_tag.is_empty() {
        // No previous tag, get all commits
        let output = Command::new("git")
            .args(["log", "--pretty=format:- %s"])
            .output()?;
        return Ok(String::from_utf8(output.stdout)?);
    }

    let output = Command::new("git")
        .args([
            "log",
            "--pretty=format:- %s",
            &format!("{}..HEAD", previous_tag),
        ])
        .output()?;

    Ok(String::from_utf8(output.stdout)?)
}

fn get_latest_tag() -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(["describe", "--tags", "--abbrev=0"])
        .output()?;

    if !output.status.success() {
        // No tags exist yet, return a default value or initial commit
        let initial_output = Command::new("git")
            .args(["rev-list", "--max-parents=0", "HEAD"])
            .output()?;

        if initial_output.status.success() {
            return Ok(String::from_utf8(initial_output.stdout)?.trim().to_string());
        }

        // Fallback to empty string if even that fails
        return Ok(String::new());
    }

    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}

fn confirm(message: &str) -> Result<bool, io::Error> {
    print!("{} (y/n): ", message);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    Ok(input.trim().to_lowercase() == "y")
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read current Cargo.toml
    let cargo_content = fs::read_to_string("Cargo.toml")?;
    let mut doc = cargo_content.parse::<DocumentMut>()?;

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

    if new_version.is_empty() {
        return Err("Version cannot be empty".into());
    }

    // Confirm release
    if !confirm(&format!("Ready to release version {}?", new_version))? {
        println!("Release aborted.");
        return Ok(());
    }

    // Update Cargo.toml
    doc["package"]["version"] = Item::from(new_version);
    fs::write("Cargo.toml", doc.to_string())?;
    println!("Updated Cargo.toml with new version: {}", new_version);

    // Update Cargo.lock to match the new version
    println!("Updating Cargo.lock...");
    let status = Command::new("cargo").arg("check").status()?;
    if !status.success() {
        return Err("Failed to update Cargo.lock".into());
    }

    // Get the latest tag for commit history
    let previous_tag = get_latest_tag()?;
    println!(
        "Previous tag: {}",
        if previous_tag.is_empty() {
            "None"
        } else {
            &previous_tag
        }
    );

    let commit_history = get_commit_history(&previous_tag)?;
    if commit_history.is_empty() {
        println!("Warning: No commit history found between previous tag and HEAD.");
        if !confirm("Continue with empty release notes?")? {
            println!("Release aborted.");
            return Ok(());
        }
    } else {
        println!("Commit history for release notes:");
        println!("{}", commit_history);
    }

    // Git commands
    let commands = [
        (
            "git add Cargo.toml Cargo.lock",
            "Failed to stage Cargo.toml",
        ),
        (
            &format!("git commit -m \"Bump version to {}\"", new_version),
            "Failed to commit version bump",
        ),
        (
            &format!("git tag -a v{} -m \"Version {}\"", new_version, new_version),
            "Failed to create tag",
        ),
        ("git push", "Failed to push commits"),
        ("git push --tags", "Failed to push tags"),
    ];

    for (cmd, error_msg) in commands.iter() {
        println!("Executing: {}", cmd);
        let status = Command::new("sh").arg("-c").arg(cmd).status()?;

        if !status.success() {
            return Err(error_msg.to_string().into());
        }
    }

    // Confirm publishing to crates.io
    if confirm("Publish to crates.io?")? {
        println!("Publishing to crates.io...");
        let status = Command::new("cargo").arg("publish").status()?;

        if !status.success() {
            return Err("Failed to publish to crates.io".into());
        }
    } else {
        println!("Skipping crates.io publishing.");
    }

    // Create GitHub release
    if confirm("Create GitHub release?")? {
        println!("Creating GitHub release...");
        let create_release = Command::new("gh")
            .args([
                "release",
                "create",
                &format!("v{}", new_version),
                "--title",
                &format!("v{}", new_version),
                "--notes",
                &commit_history,
            ])
            .status()?;

        if !create_release.success() {
            return Err("Failed to create GitHub release".into());
        }
    } else {
        println!("Skipping GitHub release creation.");
    }

    println!("Successfully released version {}", new_version);
    Ok(())
}
