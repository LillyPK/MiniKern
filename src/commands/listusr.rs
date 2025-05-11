use crate::auth::load_users; // Removed `self,`

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let users = load_users()?;

    if users.is_empty() {
        println!("No users found.");
        return Ok(());
    }

    println!("User List");
    for (i, user) in users.iter().enumerate() {
        let prefix = if i == users.len() - 1 { "└──" } else { "├──" };
        println!(
            "{} {} ({})",
            prefix,
            user.username,
            if user.is_admin { "Admin" } else { "User" }
        );
    }
    Ok(())
}
