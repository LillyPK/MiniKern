use crate::auth::{
    self, hash_password, load_users, save_users, CurrentUser,
};
use crate::commands;
use std::io::{self, Write};

/// Result of the chusr command that indicates what was changed
#[derive(Debug)]
pub enum ChusrResult {
    /// No changes were made
    NoChange,
    /// Password was changed for the specified user
    PasswordChanged(String),
    /// Admin status was changed for the specified user
    AdminChanged(String),
    /// Both password and admin status were changed for the specified user
    BothChanged(String),
}

pub fn run(
    current_user: &CurrentUser,
) -> Result<ChusrResult, Box<dyn std::error::Error>> {
    println!("Modify User");
    println!("-----------------------------");
    
    // Show the current user list
    commands::listusr::run()?;

    print!("Enter Username: > ");
    io::stdout().flush()?;
    let mut username_to_change = String::new();
    io::stdin().read_line(&mut username_to_change)?;
    let username_to_change = username_to_change.trim();

    if username_to_change.is_empty() {
        println!("Username cannot be empty.");
        return Ok(ChusrResult::NoChange);
    }

    let mut users = load_users()?;
    let user_index = users
        .iter()
        .position(|u| u.username == username_to_change);

    match user_index {
        Some(index) => {
            let is_self = username_to_change == current_user.username;
            let is_root = index == 0;  // First user is root
            let is_current_root = current_user.username == users[0].username;
            let mut password_changed = false;
            let mut admin_changed = false;
            
            // Permission checks for non-admin users
            if !current_user.is_admin {
                if !is_self {
                    println!("Error: Non-admin users can only change their own password.");
                    return Ok(ChusrResult::NoChange);
                }
                
                // Non-admin users can only change their password
                println!("Change your password:");
                let new_password = auth::get_confirmed_password("Enter New Password")?;
                users[index].password_hash = hash_password(&new_password);
                println!("Your password has been updated successfully.");
                
                save_users(&users)?;
                return Ok(ChusrResult::PasswordChanged(username_to_change.to_string()));
            }
            
            // Admin users with additional permission checks
            
            // Show current admin status
            println!(
                "User '{}' currently has {} privileges.",
                username_to_change,
                if users[index].is_admin { "ADMIN" } else { "standard" }
            );
            
            // Root user special rules
            if is_root {
                println!("Note: This is the root user. Admin status cannot be changed.");
                
                if !is_current_root {
                    println!("Error: Only the root user can change root's password.");
                    return Ok(ChusrResult::NoChange);
                }
                
                // Root can change its own password
                print!("Change root password? (y/n): > ");
                io::stdout().flush()?;
                let mut password_change_input = String::new();
                io::stdin().read_line(&mut password_change_input)?;
                
                if password_change_input.trim().eq_ignore_ascii_case("y") {
                    let new_password = auth::get_confirmed_password("Enter New Password")?;
                    users[index].password_hash = hash_password(&new_password);
                    password_changed = true;
                    println!("Root password updated successfully.");
                }
                
                // Save changes if password was changed
                if password_changed {
                    save_users(&users)?;
                    return Ok(ChusrResult::PasswordChanged(username_to_change.to_string()));
                } else {
                    return Ok(ChusrResult::NoChange);
                }
            } else {
                // Regular admin changing a non-root user
                
                // Ask if admin status should be changed
                print!("Change admin privileges? (y/n): > ");
                io::stdout().flush()?;
                let mut admin_change_input = String::new();
                io::stdin().read_line(&mut admin_change_input)?;
                
                if admin_change_input.trim().eq_ignore_ascii_case("y") {
                    let new_admin_status = !users[index].is_admin;
                    
                    // Safety checks
                    if !new_admin_status { // Demoting from admin
                        // Check if trying to demote self
                        if is_self {
                            println!("Warning: You are removing your own admin privileges.");
                            print!("Are you sure? (y/n): > ");
                            io::stdout().flush()?;
                            let mut confirm = String::new();
                            io::stdin().read_line(&mut confirm)?;
                            if !confirm.trim().eq_ignore_ascii_case("y") {
                                println!("Admin privilege change cancelled.");
                                // Continue to password section
                            } else {
                                // Count how many admins would be left
                                let admin_count = users.iter()
                                    .filter(|u| u.is_admin && u.username != username_to_change)
                                    .count();
                                    
                                if admin_count == 0 {
                                    println!("Error: Cannot remove the last admin user.");
                                    println!("Create another admin user first.");
                                    // Continue to password section
                                } else {
                                    users[index].is_admin = new_admin_status;
                                    admin_changed = true;
                                    println!(
                                        "User '{}' admin privileges have been removed.",
                                        username_to_change
                                    );
                                }
                            }
                        } else {
                            // Count how many admins would be left
                            let admin_count = users.iter()
                                .filter(|u| u.is_admin && u.username != username_to_change)
                                .count();
                                
                            if admin_count == 0 {
                                println!("Error: Cannot remove the last admin user.");
                                println!("Create another admin user first.");
                                // Continue to password section
                            } else {
                                users[index].is_admin = new_admin_status;
                                admin_changed = true;
                                println!(
                                    "User '{}' admin privileges have been removed.",
                                    username_to_change
                                );
                            }
                        }
                    } else { // Promoting to admin
                        users[index].is_admin = new_admin_status;
                        admin_changed = true;
                        println!(
                            "User '{}' has been granted admin privileges.",
                            username_to_change
                        );
                    }
                }
                
                // Ask if password should be changed
                print!("Change password? (y/n): > ");
                io::stdout().flush()?;
                let mut password_change_input = String::new();
                io::stdin().read_line(&mut password_change_input)?;
                
                if password_change_input.trim().eq_ignore_ascii_case("y") {
                    let new_password =
                        auth::get_confirmed_password("Enter New Password")?;
                    users[index].password_hash = hash_password(&new_password);
                    password_changed = true;
                    println!(
                        "Password for '{}' updated successfully.",
                        username_to_change
                    );
                }
                
                // Save changes if any were made
                if password_changed || admin_changed {
                    save_users(&users)?;
                    
                    // Return appropriate result based on what changed
                    if password_changed && admin_changed {
                        return Ok(ChusrResult::BothChanged(username_to_change.to_string()));
                    } else if password_changed {
                        return Ok(ChusrResult::PasswordChanged(username_to_change.to_string()));
                    } else {
                        return Ok(ChusrResult::AdminChanged(username_to_change.to_string()));
                    }
                } else {
                    println!("No changes were made to user '{}'.", username_to_change);
                    return Ok(ChusrResult::NoChange);
                }
            }
        }
        None => {
            println!("User '{}' not found.", username_to_change);
            return Ok(ChusrResult::NoChange);
        }
    }
}
