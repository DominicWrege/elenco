use crate::{Client, model::Permission};

use super::error::{AuthError, ValidationError};

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterForm {
    username: String,
    email: String,
    password: String,
    password_check: String,
}

pub async fn new_account(
    client: &mut Client,
    form: &RegisterForm,
    permission: Permission,
) -> Result<(), AuthError> {
    let trx = client.transaction().await?;
    let pwd_hash =
        bcrypt::hash(&form.password, 8).map_err(|err| AuthError::Internal(err.into()))?;

    let stmt = trx
        .prepare("INSERT INTO Account(username, password_hash, email, account_type) Values($1, $2, $3, $4)")
        .await?;
    trx.execute(
        &stmt,
        &[&form.username, &pwd_hash, &form.email, &permission],
    )
    .await
    .map_err(|_e| AuthError::EmailOrUsernameExists)?;
    trx.commit().await?;
    Ok(())
}

pub fn validate_register_form(form: &RegisterForm) -> Result<(), ValidationError> {
    let pwd_len = 4;
    let RegisterForm {
        email,
        password,
        password_check,
        ..
    } = form;

    match (email, password, password_check) {
        (_, password, password_check) if password != password_check => {
            Err(ValidationError::PasswordMisMatch)
        }
        (email, _, _) if email_address::EmailAddress::is_valid(&email) == false => {
            Err(ValidationError::InvalidEmail)
        }
        (_, password, password_check)
            if password.len() < pwd_len || password_check.len() < pwd_len =>
        {
            Err(ValidationError::PasswordShort(password.clone()))
        }
        _ => Ok(()),
    }
}
