use crate::auth::CurrentUser; // Removed `self,`
use crate::commands;
use std::io::{self, Write};

pub fn run_terminal(
    current_user: CurrentUser,
) -> Result<bool, Box<dyn std::error::Error>> {
    println!("-----------------------------");
    println!("Welcome, {}!", current_user.username);
    if current_user.is_admin {
        println!("You have ADMIN privileges.");
    }
    println!("Type 'help' for available commands, 'exit' to quit.");

    loop {
        print!("{}> ", current_user.username);
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let command = input.trim().to_lowercase();

        if command.is_empty() {
            continue;
        }

        println!("-----------------------------");

        match command.as_str() {
            "addusr" => {
                if current_user.is_admin {
                    if let Err(e) = commands::addusr::run(&current_user) {
                        eprintln!("Error adding user: {}", e);
                    }
                } else {
                    println!(
                        "Error: You must be an admin to add users."
                    );
                }
            }
            "listusr" => {
                if let Err(e) = commands::listusr::run() {
                    eprintln!("Error listing users: {}", e);
                }
            }
            "delusr" => {
                if current_user.is_admin {
                    match commands::delusr::run(&current_user) {
                        Ok(result) => {
                            match result {
                                commands::delusr::DeleteResult::NoDelete => {
                                    // Nothing to do
                                }
                                commands::delusr::DeleteResult::OtherUserDeleted(username) => {
                                    println!("User '{}' was successfully deleted.", username);
                                }
                                commands::delusr::DeleteResult::CurrentUserDeleted => {
                                    println!("Your account has been deleted. Logging out.");
                                    return Ok(false); // Trigger logout
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Error deleting user: {}", e);
                        }
                    }
                } else {
                    println!("Error: You must be logged in as an admin to delete users.");
                }
            }
            "chusr" => {
                match commands::chusr::run(&current_user) {
                    Ok(result) => {
                        match result {
                            commands::chusr::ChusrResult::NoChange => {
                                // Nothing to do
                            }
                            commands::chusr::ChusrResult::PasswordChanged(username) => {
                                if username == current_user.username {
                                    println!("Your password has changed. Please log in again.");
                                    return Ok(false); // Trigger logout
                                }
                            }
                            commands::chusr::ChusrResult::AdminChanged(username) => {
                                if username == current_user.username {
                                    println!("Your admin status has changed. Please log in again.");
                                    return Ok(false); // Trigger logout
                                }
                            }
                            commands::chusr::ChusrResult::BothChanged(username) => {
                                if username == current_user.username {
                                    println!("Your account has been modified. Please log in again.");
                                    return Ok(false); // Trigger logout
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Error changing user: {}", e);
                    }
                }
            }
            "help" => {
                println!("Available commands:");
                println!("  addusr    - Add a new user (admin only)");
                println!("  listusr   - List all users");
                println!(
                    "  chusr     - Change user passwords and admin status (users can change their own password)"
                );
                println!("  delusr    - Delete a user (admin only)");
                println!("  logout    - Log out and login as another user");
                println!("  exit      - Log out and exit the program");
                println!("  help      - Show this help message");
            }
            "logout" => {
                println!("Logging out. Please log in again.");
                return Ok(false);
            }
            "exit" => {
                println!("Logging out. Goodbye!");
                return Ok(true);
            }
            _ => {
                println!("Unknown command: '{}'. Type 'help' for a list of commands.", command);
            }
        }
        if command != "exit" {
             println!("-----------------------------");
        }
    }
}
