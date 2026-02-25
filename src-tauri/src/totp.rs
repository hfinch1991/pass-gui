use crate::types::TotpCode;

pub fn generate_code(otpauth_uri: &str) -> Result<TotpCode, String> {
    let totp = totp_rs::TOTP::from_url(otpauth_uri)
        .map_err(|e| format!("Failed to parse TOTP URI: {}", e))?;

    let time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| format!("Failed to get system time: {}", e))?;

    let seconds = time.as_secs();
    let period = totp.step;
    let seconds_remaining = period - (seconds % period);

    let code = totp
        .generate(seconds)
        .to_string();

    Ok(TotpCode {
        code,
        seconds_remaining,
        period,
    })
}
