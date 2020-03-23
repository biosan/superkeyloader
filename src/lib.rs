#[macro_use]
pub extern crate log;

#[macro_use]
pub extern crate serde_derive;

pub use exitfailure::ExitDisplay;
pub use failure::ResultExt;

pub mod github;

pub use github as gh;

///
/// Handle HTTP status codes errors and "no SSH keys" error.
///
/// TODO: Add context about username and service to improve error messages
///
/// # Errors
///
/// The function use the crate `exitfailure` to print a better error message on exit.
/// It returns an error string that contains the error description on:
///   - Input vector length is 0
///   - Input has no `Ok` result, but has error code:
///     - `404` HTTP Status Code -> Usually it means that the user doesn't exists
///     - `1001` Internal error -> GitHub username is invalid
///         (see https://github.com/shinnn/github-username-regex)
///     - `1002` Internal error -> GitHub API response could not be parsed
///     - Every other status code -> Unrecognized HTTP status code (i.e. 500, 501, etc.)
///
/// > Assuming that a 2XX response will always have an `Ok` value, so it will never reach
/// > 'error matching'
///
///
/// # Examples
///
/// ## No errors
///
/// ```
/// use superkeyloader_lib::error_handler_wrapper;
///
/// let data = vec!("key1".to_string(), "key1".to_string());
/// let input = Ok(data.clone());
/// let output = error_handler_wrapper(input);
///
/// assert_eq!(data, output.unwrap());
/// ```
///
/// ## Missing user (404 status code)
///
/// ```
/// use superkeyloader_lib::error_handler_wrapper;
///
/// let error_code: u16 = 404;
/// let input = Err(error_code);
/// let output = error_handler_wrapper(input);
///
/// assert!(output.is_err());
///
/// let expected_output = String::from("Wrong username");
/// let error_message = output.err().unwrap();
///
/// assert!(error_message.contains(&expected_output));
/// ```
///
pub fn error_handler_wrapper(res: Result<Vec<String>, u16>) -> Result<Vec<String>, String> {
    match res {
        Ok(res) => match res.len() {
            0 => Err("User has no SSH keys available".into()),
            _ => Ok(res),
        },
        Err(err) => match err {
            404 => Err("Wrong username, doesn't exists".into()),
            gh::INVALID_GH_API_RESPONSE => Err("Invalid GitHub API response".into()),
            gh::INVALID_GH_USERNAME => {
                Err(format!(
                    "Invalid username. Username isn't allowed on GitHub. \
                            If you think this is an bug, please create a issue on at {}/issues",
                    env!("CARGO_PKG_REPOSITORY")
                )) // TODO: Maybe add this message to all error infos?
            }
            _ => Err(format!("API response code: {}", err)),
        },
    }
}

//
// Testing
//

#[test]
fn test_error_handling() {
    // All Ok
    let all_ok_input: Result<Vec<String>, u16> = Ok(vec!["key1".to_string(), "key2".to_string()]);
    let all_ok_output: Result<Vec<String>, failure::Error> =
        Ok(vec!["key1".to_string(), "key2".to_string()]);
    assert_eq!(
        error_handler_wrapper(all_ok_input).unwrap(),
        all_ok_output.unwrap()
    );

    // No keys
    let no_keys_input: Result<Vec<String>, u16> = Ok(vec![]);
    assert_eq!(error_handler_wrapper(no_keys_input).is_err(), true);

    // No user
    let no_user_error_code: u16 = 404;
    let no_user_input: Result<Vec<String>, u16> = Err(no_user_error_code);
    assert_eq!(error_handler_wrapper(no_user_input).is_err(), true);

    // Other error
    let other_error_code: u16 = 500;
    let other_error_input: Result<Vec<String>, u16> = Err(other_error_code);
    assert_eq!(error_handler_wrapper(other_error_input).is_err(), true);

    //
    // GitHub Errors
    //

    // Invalid GitHub username
    let invalid_user_input: Result<Vec<String>, u16> = Err(gh::INVALID_GH_USERNAME);
    assert_eq!(error_handler_wrapper(invalid_user_input).is_err(), true);

    // Invalid GitHub API Response
    let invalid_user_input: Result<Vec<String>, u16> = Err(gh::INVALID_GH_API_RESPONSE);
    assert_eq!(error_handler_wrapper(invalid_user_input).is_err(), true);
}
