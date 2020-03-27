extern crate pretty_env_logger;

use regex::RegexSet;

pub const INVALID_GH_USERNAME: u16 = 1001;
pub const INVALID_GH_API_RESPONSE: u16 = 1002;

///
/// GitHub API response parsing struct (REST v3)
///
/// [Documentation](https://developer.github.com/v3/users/keys/)
///
/// URL: `GET https://api.github.com/user/<USERNAME>/keys`
///
/// # Example
///
/// ```
/// use superkeyloader_lib::github::GhKey;
///
/// let json_string = r#"
///   [
///     {
///       "id": 12257919,
///       "key": "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQCarT/me5sWxY9Tizc"
///     },
///     {
///       "id": 22932337,
///       "key": "ssh-rsa AAAAB3N"
///     }
///   ]
/// "#;
/// let parsed_json = serde_json::from_str(&json_string);
/// let keys: Vec<GhKey> = parsed_json.unwrap();
///
/// assert_eq!(keys[0].id, 12257919);
/// assert_eq!(keys[1].key, "ssh-rsa AAAAB3N");
/// ```
///
#[derive(Debug, Serialize, Deserialize)]
pub struct GhKey {
    pub id: u64,
    pub key: String,
}

///
/// Regex (set) to validate GitHub usernames
///
/// Thanks to: https://github.com/shinnn/github-username-regex
///
/// # Rules
///   - Max 39 characters, alphanumerical and '-' (case insensitive)
///   - Cannot not end with '-'
///   - Cannot have multiple consecutive hyphens
///
/// > with look-arounds a single regex (no set) could be used
///
/// TODO: Move to a single regex with look-arounds when will be supported by the standard Rust regex library.
///
fn validate_username(username: &str) -> bool {
    let username_rules = RegexSet::new(vec![
        r"^([-a-zA-Z\d]){1,39}$",
        r".*[^-]$",
        r"^([^-]+|-($[^-]))*$",
    ])
    .unwrap();

    let matches: Vec<_> = username_rules.matches(username).into_iter().collect();
    // If all rules match then the username is valid
    username_rules.len() == matches.len()
}

///
/// Download user's SSH keys from GitHub
///
/// Return a vector of `String` containing all the user keys in the exact same order they were send
/// by the API.
///
/// Output keys format is the following:
/// `<SSH_KEY> from-GH-id-<KEY_ID>`
///
/// > `KEY_ID` is the internal GitHub key id.
///
/// # Errors
///
/// Return the response status code if it's not a 2XX status code.
/// Return an internal error code:
///   - `1001` if GitHub username isn't valid
///     code stored in `INVALID_GH_USERNAME`
///   - `1002` if GitHub API response could not be parsed
///     code stored in `INVALID_GH_API_RESPONSE`
///
/// # Example
///
/// ```
/// let token: Option<String>;
/// # token = std::env::var("GITHUB_TOKEN").ok();
/// use superkeyloader_lib::github::get_keys;
///
/// let keys = get_keys("biosan", token).unwrap();
///
/// assert!(keys[0].contains(&String::from("ssh")));
/// assert!(keys[0].contains(&String::from(" from-GH-id-")));
/// ```
///
pub fn get_keys(username: &str, token: Option<String>) -> Result<Vec<String>, u16> {
    if !validate_username(username) {
        return Err(INVALID_GH_USERNAME);
    }

    // TODO: I don't like very much this approach... find a better way
    #[cfg(not(test))]
    let gh_api_url: &str = "https://api.github.com";
    #[cfg(test)]
    let gh_api_url: &str = &mockito::server_url();
    debug!("GitHub API base URL: {}", gh_api_url);

    // 1. Make HTTP request
    // 2. Transmform reponse JSON to an array of keys
    let url = format!("{}/users/{}/keys", gh_api_url, username);
    debug!("GitHub API endpoint URL: {}", url);

    let mut request = ureq::get(&url);

    if let Some(oauth_token) = token {
        request.set("Authorization", format!("token {}", oauth_token).as_ref());
    }

    let response = request.call();

    if !response.ok() {
        return Err(response.status());
    }

    let resp_json = response.into_string().unwrap();
    let parsed_json = serde_json::from_str(&resp_json);

    if parsed_json.is_err() {
        return Err(INVALID_GH_API_RESPONSE);
    }

    let gh_keys: Vec<GhKey> = parsed_json.unwrap();

    let keys = gh_keys
        .into_iter()
        .map(|key| format!("{} from-GH-id-{}", key.key, key.id))
        .collect();

    Ok(keys)
}

pub mod test_values {

    pub const VALID_USERNAME: &str = "testuser";
    pub const MISSING_USERNAME: &str = "erruser";
    pub const INVALID_USERNAME_LENGTH: &str = "user-user-user-user-user-user-user-user-";
    pub const INVALID_USERNAME_ENDING_HYPHEN: &str = "user-user-";
    pub const INVALID_USERNAME_CONSEC_HYPHEN: &str = "user--user";

    pub const VALID_3_KEYS_JSON: &str = r#"[
      {
        "id": 12257919,
        "key": "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQCarT/me5sWxY9Tizc"
      },
      {
        "id": 22932337,
        "key": "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAACAQC+MxvBji8iUuN2so2"
      },
      {
        "id": 69196823,
        "key": "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQDq/BrJT0c7LSmTRDE"
      }
    ]"#;

    pub const EMPTY_JSON: &str = r#"[]"#;

    pub const INVALID_JSON: &str = r#"[
      {
        "id": "12257919",
        "key": "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQCarT/me5sWxY9Tizc"
      },
      {
        "key": "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAACAQC+MxvBji8iUuN2so2"
      },
      {
        "id": 69196823,
        "key": 42
      }
    ]"#;
}

#[cfg(test)]
mod tests {

    use super::test_values::*;

    use mockito::mock;

    #[test]
    fn test_github_username_validation() {
        assert_eq!(
            super::validate_username(&String::from(VALID_USERNAME)),
            true
        );
        assert_eq!(
            super::validate_username(&String::from(INVALID_USERNAME_LENGTH)),
            false
        );
        assert_eq!(
            super::validate_username(&String::from(INVALID_USERNAME_ENDING_HYPHEN)),
            false
        );
        assert_eq!(
            super::validate_username(&String::from(INVALID_USERNAME_CONSEC_HYPHEN)),
            false
        );
    }

    #[test]
    fn valid_response() {
        let _m = mock("GET", "/users/testuser/keys")
            .with_status(200)
            .with_header("Content-Type", "application/json; charset=utf-8")
            .with_body(VALID_3_KEYS_JSON)
            .create();

        let result = super::get_keys(&String::from(VALID_USERNAME), None);

        assert_eq!(result.is_ok(), true);
        assert_eq!(result.unwrap().len(), 3);
    }

    #[test]
    fn invalid_response() {
        let _m = mock("GET", "/users/testuser/keys")
            .with_status(200)
            .with_header("Content-Type", "application/json; charset=utf-8")
            .with_body(INVALID_JSON)
            .create();

        let result = super::get_keys(&String::from(VALID_USERNAME), None);

        assert_eq!(result.is_ok(), false);
        assert_eq!(result.err().unwrap(), super::INVALID_GH_API_RESPONSE);
    }

    #[test]
    fn no_keys_response() {
        let _m = mock("GET", "/users/testuser/keys")
            .with_status(200)
            .with_header("Content-Type", "application/json; charset=utf-8")
            .with_body(EMPTY_JSON)
            .create();

        let result = super::get_keys(&String::from(VALID_USERNAME), None);

        assert_eq!(result.is_ok(), true);
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn missing_username() {
        let _m = mock("GET", "/users/erruser/keys")
            .with_status(404)
            .with_header("Content-Type", "application/json; charset=utf-8")
            .with_body(VALID_3_KEYS_JSON)
            .create();

        let result = super::get_keys(&String::from(MISSING_USERNAME), None);

        assert_eq!(result.is_ok(), false);
        assert_eq!(result.err().unwrap(), 404);
    }

    #[test]
    fn invalid_username() {
        let _m = mock("GET", "/users/testuser/keys")
            .with_status(200)
            .with_header("Content-Type", "application/json; charset=utf-8")
            .with_body(VALID_3_KEYS_JSON)
            .create();

        // Test 'too long' username case
        let result = super::get_keys(&String::from(INVALID_USERNAME_LENGTH), None);
        assert_eq!(result.is_ok(), false);
        assert_eq!(result.err().unwrap(), super::INVALID_GH_USERNAME);

        // Test 'ending with hyphen' username case
        let result = super::get_keys(&String::from(INVALID_USERNAME_ENDING_HYPHEN), None);
        assert_eq!(result.is_ok(), false);
        assert_eq!(result.err().unwrap(), super::INVALID_GH_USERNAME);

        // Test 'two consecutive' username case
        let result = super::get_keys(&String::from(INVALID_USERNAME_CONSEC_HYPHEN), None);
        assert_eq!(result.is_ok(), false);
        assert_eq!(result.err().unwrap(), super::INVALID_GH_USERNAME);
    }
}
