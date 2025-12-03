use regex::Regex;
use ureq;

use crate::errors::{AppError, AppResult};

pub struct Dependency {
    pub id: String,
    pub name: String,
}

pub fn fetch_dependencies(workshop_id: &str) -> AppResult<Vec<Dependency>> {
    let url = format!(
        "https://steamcommunity.com/sharedfiles/filedetails/?id={}",
        workshop_id
    );

    let body = ureq::get(&url)
        .call()
        .map_err(|e| AppError::NetworkError(e.to_string()))?
        .into_body()
        .read_to_string()
        .map_err(|e| AppError::NetworkError(e.to_string()))?;

    // Extract the "RequiredItems" section to avoid false positives
    // The section starts with id="RequiredItems" and ends... well, it closes the div.
    // Simpler regex: look for the specific link pattern which usually appears inside that block.
    // But we should be careful not to pick up other links.
    // The pattern is `<a href="...id=(\d+)" ... ><div class="requiredItem">\s*(.*?)\s*</div></a>`

    // Let's first verify the "RequiredItems" container exists
    if !body.contains("id=\"RequiredItems\"") {
        return Ok(Vec::new());
    }

    let mut dependencies = Vec::new();

    // Regex to capture ID and Name
    // Matches: <a href="...id=12345"...> ... <div class="requiredItem">Mod Name</div> ... </a>
    // We use dot_matches_new_line for the content between tags
    let re = Regex::new(
        r#"<a href="[^"]+id=(\d+)"[^>]*>\s*<div class="requiredItem">\s*(.*?)\s*</div>\s*</a>"#,
    )
    .map_err(|_| AppError::RegexError)?;

    for cap in re.captures_iter(&body) {
        if let (Some(id), Some(name)) = (cap.get(1), cap.get(2)) {
            dependencies.push(Dependency {
                id: id.as_str().to_string(),
                name: name.as_str().trim().to_string(),
            });
        }
    }

    Ok(dependencies)
}
