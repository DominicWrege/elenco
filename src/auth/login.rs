use super::error::ValidationError;

#[derive(Debug, serde::Deserialize)]
pub struct LoginForm {
    pub password: String,
    pub email: String,
}

pub fn validate_login_form(form: &LoginForm) -> Result<(), ValidationError> {
    match (&form.password, &form.email) {
        (_, email) if email_address::EmailAddress::is_valid(&email) == false => {
            Err(ValidationError::InvalidEmail)
        }
        (password, _) if password.is_empty() => {
            Err(ValidationError::PasswordShort(password.clone()))
        }
        _ => Ok(()),
    }
}
