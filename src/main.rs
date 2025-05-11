mod auth;
mod commands;
mod terminal;

use auth::{
    hash_password, load_users, save_users, CurrentUser, User,
    USERS_FILE_PATH,
};
use std::io::{self, Write};
use std::path::Path;

fn initial_user_setup() -> Result<(), Box<dyn std::error::Error>> {
    println!("MiniKern OS Loaded");
    println!("-----------------------------");
    println!("No users found or user file is invalid.");
    println!("\nCreate a new admin user");
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
        break name;
    };

    let password = auth::get_confirmed_password("Password")?;
    let password_hash = hash_password(&password);

    let admin_user = User {
        username: username.clone(),
        password_hash,
        is_admin: true, // First user is always admin
    };

    save_users(&[admin_user])?;
    println!("Admin user '{}' created successfully.", username);
    Ok(())
}

fn login_procedure(
    users: &[User],
) -> Result<CurrentUser, Box<dyn std::error::Error>> {
    println!("-----------------------------");
    println!("Login");
    for _attempt in 0..3 { // Allow 3 login attempts
        print!("Username: > ");
        io::stdout().flush()?;
        let mut username_input = String::new();
        io::stdin().read_line(&mut username_input)?;
        let username_input = username_input.trim();

        let password_input =
            auth::prompt_password_hidden("Password: > ")?;

        if let Some(user) =
            users.iter().find(|u| u.username == username_input)
        {
            if user.password_hash == hash_password(&password_input) {
                println!("Login successful!");
                return Ok(CurrentUser {
                    username: user.username.clone(),
                    is_admin: user.is_admin,
                });
            }
        }
        println!("Invalid username or password. Please try again.");
    }
    Err("Too many failed login attempts.".into())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Check for users.xml and load users
    let initial_users = match load_users() {
        Ok(users_vec) => users_vec,
        Err(e) => {
            // If loading fails (e.g., malformed XML), treat as no users.
            // A more robust solution might offer to repair or backup.
            eprintln!(
                "Warning: Could not load users from {}: {}. Proceeding with initial setup if needed.",
                USERS_FILE_PATH, e
            );
            // Attempt to delete potentially corrupt file before creating a new one
            if Path::new(USERS_FILE_PATH).exists() {
                let _ = std::fs::remove_file(USERS_FILE_PATH);
            }
            Vec::new()
        }
    };

    // Initialize users if needed
    if initial_users.is_empty() {
        initial_user_setup()?;
    }

    let mut should_exit = false;
    
    while !should_exit {
        // Always reload users to get fresh data
        let current_users = load_users()?;
        if current_users.is_empty() {
            // This should not happen if initial_user_setup succeeded
            return Err("No users found in system.".into());
        }
        
        // Display welcome message
        println!("MiniKern OS Loaded");
        
        // Login procedure
        let current_user = login_procedure(&current_users)?;

        // Run terminal and check if user wants to exit completely
        should_exit = terminal::run_terminal(current_user)?;
    }

    Ok(())
}
