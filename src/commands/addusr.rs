use crate::auth::{
    self, hash_password, load_users, save_users, User,
};
use std::io::{self, Write};

pub fn run(
    _current_user: &auth::CurrentUser, // Assumed admin by terminal
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Create a new user");
    println!("-----------------------------");

    let username = loop {
        print!("Username: > ");
        io::stdout().flush()?;
        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer)?;
        let name = buffer.trim().to_string();
        if name.is_empty() {
            println!("Username cannot be empty.");
            continue;
        }
        if name.contains(|c: char| c.is_whitespace() || "<>&\"'".contains(c))
        {
            println!(
                "Username cannot contain spaces or XML special characters."
            );
            continue;
        }
        // Check if user already exists
        let users = load_users()?;
        if users.iter().any(|u| u.username == name) {
            println!("User '{}' already exists. Try a different username.", name);
            continue;
        }
        break name;
    };

    let password = auth::get_confirmed_password("Password")?;
    let password_hash = hash_password(&password);

    let is_admin = loop {
        print!("Grant admin privileges? (y/n): > ");
        io::stdout().flush()?;
        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer)?;
        match buffer.trim().to_lowercase().as_str() {
            "y" | "yes" => break true,
            "n" | "no" => break false,
            _ => println!("Invalid input. Please enter 'y' or 'n'."),
        }
    };

    let new_user = User {
        username: username.clone(),
        password_hash,
        is_admin,
    };

    let mut users = load_users()?;
    users.push(new_user);
    save_users(&users)?;

    println!(
        "User '{}' created{}.",
        username,
        if is_admin { " with admin privileges" } else { "" }
    );
    Ok(())
}
