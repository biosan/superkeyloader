extern crate pretty_env_logger;

use regex::RegexSet;

pub const INVALID_GITLAB_USERNAME: u16 = 1003;
pub const INVALID_GITLAB_API_RESPONSE: u16 = 1004;

///
/// Check is every line of GitLab API response is a valid SSH key
///
fn validate_response(response: &str) -> bool {
    // Split input string by 'line', it returns an iterator, apply the function 'validate_ssh_key'
    // to every 'line' and return true only if **ALL** 'line's are valid
    response
        .trim()
        .split('\n')
        .filter(|line| !line.is_empty())
        .all(|line| validate_ssh_key(&line))
}

///
/// Very basic SSH key validator
/// Only checks if:
///   - key contains only valid characters (TODO)
///   - key is composed of at least 2 parts
///   - key data is actual base64
///   - key type is one of valid algorithms
///     'ssh-rsa', 'ssh-ecdsa'
///
fn validate_ssh_key(key: &str) -> bool {
    let valid_key_types: Vec<&str> = vec!["ssh-rsa", "ssh-ecdsa"];

    // TODO: Maybe use `split_ascii_whitespace`?
    let parts: Vec<&str> = key.trim().split_whitespace().collect();

    if parts.len() < 2 {
        debug!(
            "Key 'parts' are less than 2. Input string: {} - Parts: {:?}",
            key, parts
        );
        return false;
    }

    // TODO: Find a more elegant way
    let (key_type, key_data) = (parts[0], parts[1]);

    if base64::decode(key_data).is_err() {
        debug!(
            "Key data is not base64. Input string: {} - Key data: {}",
            key, key_data
        );
        return false;
    }

    if !valid_key_types.contains(&key_type) {
        debug!(
            "Key type is not valid. Input string: {} - Input key type: {} - Valid key types: {:?}",
            key, key_type, valid_key_types
        );
        return false;
    }

    true
}

///
/// Regex (set) to validate GitLab usernames
///
/// At the moment I've not found any other rule
///
/// # Rules
///   - Alphanumerical, '-', '_', '.' (case insensitive)
///
/// TODO: Find more specific rules
///
fn validate_username(username: &str) -> bool {
    let username_rules = RegexSet::new(vec![
        // r"^[\._-a-zA-Z\d]+$"
        r"^([\.\-\w]+)$",
    ])
    .unwrap();

    let matches: Vec<_> = username_rules.matches(username).into_iter().collect();
    // If all rules match then the username is valid
    username_rules.len() == matches.len()
}

///
/// Download user's SSH keys from GitLab
///
/// Return a vector of `String` containing all the user keys in the exact same order they were send
/// by the API.
///
/// # Errors
///
/// Return the response status code if it's not a 2XX status code.
/// Return an internal error code:
///   - `1003 if GitLab username isn't valid
///     code stored in `INVALID_GH_USERNAME`
///   - `1004` if GitLab API response could not be parsed
///     code stored in `INVALID_GH_API_RESPONSE`
///
/// # Example
///
/// ```
/// let token: Option<String>;
/// # token = std::env::var("GITLAB_TOKEN").ok();
/// use superkeyloader_lib::gitlab::get_keys;
///
/// let keys = get_keys("biosan", token).unwrap();
///
/// assert!(keys[0].contains(&String::from("ssh")));
/// assert!(keys[0].contains(&String::from("gitlab.com")));
/// ```
///
pub fn get_keys(username: &str, token: Option<String>) -> Result<Vec<String>, u16> {
    if !validate_username(username) {
        return Err(INVALID_GITLAB_USERNAME);
    }

    // TODO: I don't like very much this approach... find a better way
    #[cfg(not(test))]
    let gitlab_api_url: &str = "https://gitlab.com";
    #[cfg(test)]
    let gitlab_api_url: &str = &mockito::server_url();
    debug!("GitLab API base URL: {}", gitlab_api_url);

    // 1. Make HTTP request
    // 2. Transmform reponse JSON to an array of keys
    let url = format!("{}/{}.keys", gitlab_api_url, username);
    debug!("GitLab API endpoint URL: {}", url);

    let mut request = ureq::get(&url);

    if let Some(oauth_token) = token {
        // OAuth compliat headers support both OAuth tokens and personal tokens.
        // You will probably use personal tokens.
        // See https://docs.gitlab.com/ee/api/#personal-access-tokens
        request.set("Authorization", format!("Bearer {}", oauth_token).as_ref());
    }

    let response = request.call();

    if !response.ok() {
        return Err(response.status());
    }

    let response = response.into_string().unwrap();

    if !validate_response(&response) {
        return Err(INVALID_GITLAB_API_RESPONSE);
    }

    let keys = response
        .trim()
        .split('\n')
        .filter(|line| !line.is_empty())
        .map(String::from)
        .collect();

    Ok(keys)
}

pub mod test_values {

    pub const VALID_USERNAME: &str = "test_1.user-name";
    pub const MISSING_USERNAME: &str = "erruser";
    pub const INVALID_USERNAME_CHARS: &str = "user!user";

    pub const VALID_3_KEYS_STRING: &str = r#"
        ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQCarT/me5sWxY9Tizc+2SEtJLzqJuSLQWXycIiJc9azQCsPqqLiYt1ge3Df0ctpYJqUfrR1UQ7KOOVR3i78dxyPS9PxqXorGtkl7K7BAeI08nBPICYFExusbz3YqudEU9+KKK7STwvDH8O+EU/UTWlQMvsYj4JaKNU40HJTc2yWO+k12Xe3p2Zhl3TTPaJkQfJnlATX6r6LoT1aQAUnuyjpaGCWjGHSU4lBUhESPvPArZW4k9fMM4/eb7TZS5szU0GXi4gWjMpdPMdpdzksZoXQV07A7X6ZFtLTkpVAWw7i88BVC/IRC+Bl/NVPuRZsC0wW+t+tzFqhud0ZiMEx4UHh
        ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQCarT/me5sWxY9Tizc+2SEtJLzqJuSLQWXycIiJc9azQCsPqqLiYt1ge3Df0ctpYJqUfrR1UQ7KOOVR3i78dxyPS9PxqXorGtkl7K7BAeI08nBPICYFExusbz3YqudEU9+KKK7STwvDH8O+EU/UTWlQMvsYj4JaKNU40HJTc2yWO+k12Xe3p2Zhl3TTPaJkQfJnlATX6r6LoT1aQAUnuyjpaGCWjGHSU4lBUhESPvPArZW4k9fMM4/eb7TZS5szU0GXi4gWjMpdPMdpdzksZoXQV07A7X6ZFtLTkpVAWw7i88BVC/IRC+Bl/NVPuRZsC0wW+t+tzFqhud0ZiMEx4UHh
        ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQCarT/me5sWxY9Tizc+2SEtJLzqJuSLQWXycIiJc9azQCsPqqLiYt1ge3Df0ctpYJqUfrR1UQ7KOOVR3i78dxyPS9PxqXorGtkl7K7BAeI08nBPICYFExusbz3YqudEU9+KKK7STwvDH8O+EU/UTWlQMvsYj4JaKNU40HJTc2yWO+k12Xe3p2Zhl3TTPaJkQfJnlATX6r6LoT1aQAUnuyjpaGCWjGHSU4lBUhESPvPArZW4k9fMM4/eb7TZS5szU0GXi4gWjMpdPMdpdzksZoXQV07A7X6ZFtLTkpVAWw7i88BVC/IRC+Bl/NVPuRZsC0wW+t+tzFqhud0ZiMEx4UHh
    "#;

    pub const EMPTY_STRING: &str = r#""#;

    pub const INVALID_STRING: &str = r#"
        ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQCarT/me5sWxY9Tizc+2SEtJLzqJuSLQWXycIiJc9azQCsPqqLiYt1ge3Df0ctpYJqUfrR1UQ7KOOVR3i78dxyPS9PxqXorGtkl7K7BAeI08nBPICYFExusbz3YqudEU9+KKK7STwvDH8O+EU/UTWlQMvsYj4JaKNU40HJTc2yWO+k12Xe3p2Zhl3TTPaJkQfJnlATX6r6LoT1aQAUnuyjpaGCWjGHSU4lBUhESPvPArZW4k9fMM4/eb7TZS5szU0GXi4gWjMpdPMdpdzksZoXQV07A7X6ZFtLTkpVAWw7i88BVC/IRC+Bl/NVPuRZsC0wW+t+tzFqhud0ZiMEx4UHh
        42
    "#;
}

#[cfg(test)]
mod tests {

    use super::test_values::*;

    use mockito::mock;

    #[test]
    fn test_gitlab_username_validation() {
        assert_eq!(
            super::validate_username(&String::from(VALID_USERNAME)),
            true
        );
        assert_eq!(
            super::validate_username(&String::from(INVALID_USERNAME_CHARS)),
            false
        );
    }

    #[test]
    fn valid_response() {
        let _m = mock("GET", "/test_1.user-name.keys")
            .with_status(200)
            .with_header("Content-Type", "application/json; charset=utf-8")
            .with_body(VALID_3_KEYS_STRING)
            .create();

        let result = super::get_keys(&String::from(VALID_USERNAME), None);

        assert_eq!(result.is_ok(), true);
        assert_eq!(result.unwrap().len(), 3);
    }

    #[test]
    fn invalid_response() {
        let _m = mock("GET", "/test_1.user-name.keys")
            .with_status(200)
            .with_header("Content-Type", "application/json; charset=utf-8")
            .with_body(INVALID_STRING)
            .create();

        let result = super::get_keys(&String::from(VALID_USERNAME), None);

        assert_eq!(result.is_ok(), false);
        assert_eq!(result.err().unwrap(), super::INVALID_GITLAB_API_RESPONSE);
    }

    #[test]
    fn no_keys_response() {
        let _m = mock("GET", "/test_1.user-name.keys")
            .with_status(200)
            .with_header("Content-Type", "application/json; charset=utf-8")
            .with_body(EMPTY_STRING)
            .create();

        let result = super::get_keys(&String::from(VALID_USERNAME), None);

        assert_eq!(result.is_ok(), true);
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn missing_username() {
        let _m = mock("GET", "/erruser.keys")
            .with_status(404)
            .with_header("Content-Type", "application/json; charset=utf-8")
            .with_body(VALID_3_KEYS_STRING)
            .create();

        let result = super::get_keys(&String::from(MISSING_USERNAME), None);

        assert_eq!(result.is_ok(), false);
        assert_eq!(result.err().unwrap(), 404);
    }

    #[test]
    fn invalid_username() {
        let _m = mock("GET", "/test_1.user-name.keys")
            .with_status(200)
            .with_header("Content-Type", "application/json; charset=utf-8")
            .with_body(VALID_3_KEYS_STRING)
            .create();

        // Test 'invalid character' username case
        let result = super::get_keys(&String::from(INVALID_USERNAME_CHARS), None);
        assert_eq!(result.is_ok(), false);
        assert_eq!(result.err().unwrap(), super::INVALID_GITLAB_USERNAME);
    }
}
