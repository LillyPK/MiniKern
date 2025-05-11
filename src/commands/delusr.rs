use crate::auth::{
    self, hash_password, load_users, save_users, CurrentUser,
};
use crate::commands;
use std::io::{self, Write};

/// Result of the delusr command
#[derive(Debug)]
pub enum DeleteResult {
    /// No user was deleted
    NoDelete,
    /// A user other than the current user was deleted
    OtherUserDeleted(String),
    /// The current user was deleted
    CurrentUserDeleted,
}

pub fn run(
    current_user: &CurrentUser,
) -> Result<DeleteResult, Box<dyn std::error::Error>> {
    println!("Delete User");
    println!("-----------------------------");
    
    // Show the current user list
    commands::listusr::run()?;
    
    // Load users
    let mut users = load_users()?;
    
    if users.len() <= 1 {
        println!("Cannot delete users when there's only one user in the system.");
        return Ok(DeleteResult::NoDelete);
    }
    
    // Prompt for username to delete
    print!("Enter username to delete: > ");
    io::stdout().flush()?;
    let mut username_to_delete = String::new();
    io::stdin().read_line(&mut username_to_delete)?;
    let username_to_delete = username_to_delete.trim();
    
    if username_to_delete.is_empty() {
        println!("Username cannot be empty.");
        return Ok(DeleteResult::NoDelete);
    }
    
    // Find the user
    let user_index = users.iter().position(|u| u.username == username_to_delete);
    
    match user_index {
        Some(index) => {
            // Check if it's the first user (index 0)
            if index == 0 {
                println!("Error: Cannot delete the first user (root admin).");
                return Ok(DeleteResult::NoDelete);
            }
            
            // Verify by asking for the root user's password
            let root_user = &users[0];
            print!("Enter password of {} (for verification): > ", root_user.username);
            io::stdout().flush()?;
            let password = auth::prompt_password_hidden("")?;
            
            // Verify password
            if hash_password(&password) != root_user.password_hash {
                println!("Incorrect password. User deletion cancelled.");
                return Ok(DeleteResult::NoDelete);
            }
            
            // Password is correct, delete the user
            let deleted_username = users[index].username.clone();
            println!("Deleting user '{}'...", deleted_username);
            users.remove(index);
            save_users(&users)?;
            
            println!("User '{}' has been deleted.", deleted_username);
            
            // Check if the current user was deleted
            if deleted_username == current_user.username {
                println!("You have deleted your own account. You will be logged out.");
                return Ok(DeleteResult::CurrentUserDeleted);
            } else {
                return Ok(DeleteResult::OtherUserDeleted(deleted_username));
            }
        }
        None => {
            println!("User '{}' not found.", username_to_delete);
            return Ok(DeleteResult::NoDelete);
        }
    }
}

