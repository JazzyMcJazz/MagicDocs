use tera::{Result, Value};

pub fn active_project(value: Option<&Value>, args: &[Value]) -> Result<bool> {
    let id = match value {
        Some(v) => match v.get("id") {
            Some(v) => v.to_string(),
            None => return Ok(false),
        },
        None => return Ok(false),
    };

    let path = match args.first() {
        Some(v) => v.to_string().replace('"', ""),
        None => return Ok(false),
    };

    let path_parts: Vec<&str> = path.split('/').collect();
    let project_path = path_parts.get(1).unwrap_or(&"");
    let project_id = path_parts.get(2).unwrap_or(&"");

    if project_path.eq(&"projects") && project_id.eq(&id) {
        return Ok(true);
    }

    Ok(false)
}
