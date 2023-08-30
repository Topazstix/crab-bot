/*
 * Modular Database Storage/API Storage
 */

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use std::io::Result;
use std::fs;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Enrollment {
    user_id: u64, 
    user_name: String,
    name: String,
    university: String,
    email: String,
    interests: String,
    email_distro: String,
}

pub fn save_to_json(enrollment: &Enrollment) -> Result<()> {
    let mut enrollments: HashMap<u64, Enrollment> = HashMap::new();

    // Load existing data
    if let Ok(data) = fs::read_to_string("enrollments.json") {
        enrollments = serde_json::from_str(&data)?;
    }

    // Add new enrollment
    enrollments.insert(enrollment.user_id, enrollment.clone());

    // Save to file
    let data = serde_json::to_string(&enrollments)?;
    fs::write("enrollments.json", data)?;

    Ok(())
}// end save_to_json