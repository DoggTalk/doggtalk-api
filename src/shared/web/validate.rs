use validator::ValidationError;

pub fn validate_url(value: &str) -> Result<(), ValidationError> {
    if value.is_empty() || value.starts_with("https://") || value.starts_with("https://") {
        return Ok(());
    }

    Err(ValidationError::new("require http or https prefix"))
}

pub fn validate_gender(value: i8) -> Result<(), ValidationError> {
    if value >= 0 && value <= 2 {
        return Ok(());
    }

    Err(ValidationError::new("between 0 to 2"))
}

pub fn validate_page_count(value: u32) -> Result<(), ValidationError> {
    if value <= 500 {
        return Ok(());
    }

    Err(ValidationError::new("less then 500"))
}
