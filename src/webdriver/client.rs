
use reqwest;

mod errors {
    error_chain! {
        foreign_links {
            Reqwest(::reqwest::Error);
        }
    }
}

use self::errors::*;

pub struct Session {
    id: String,
    client: reqwest::Client,
}

use std::collections::HashMap;

// session creation response: {"value": {"sessionId":"9f9c6fdb-ea4a-41fd-84a3-fd54b9b1c58b","capabilities":{"acceptInsecureCerts":false,"browserName":"firefox","browserVersion":"59.0.1","moz:accessibilityChecks":false,"moz:headless":false,"moz:processID":29954,"moz:profile":"/tmp/rust_mozprofile.oH01UKhZUOAL","moz:useNonSpecCompliantPointerOrigin":false,"moz:webdriverClick":true,"pageLoadStrategy":"normal","platformName":"linux","platformVersion":"4.4.0-89-generic","rotatable":false,"timeouts":{"implicit":0,"pageLoad":300000,"script":30000}}}}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct NewSessionValue {
    session_id: String,
}

#[derive(Deserialize)]
struct NewSession {
    value: NewSessionValue,
}

impl Session {
    pub fn new() -> Result<Session> {
        let client = reqwest::Client::new();
        let body: HashMap<String, String> = HashMap::new();
        
        let response: NewSession = client.post("http://localhost:4444/session").json(&body).send()?.json()?;
        let session = Session{
            client: client,
            id: response.value.session_id,
        };

        Ok(session)
    }

    pub fn delete(&self) -> Result<bool> {
        let response = self.client.delete(format!("http://localhost:4444/session/{}", self.id).as_str()).send()?.text()?;
        println!("delete response: {}", response);
        Ok(true)
    }
}

#[derive(Deserialize)]
struct Value {
    ready: bool,
}

#[derive(Deserialize)]
struct Status {
    value: Value, 
}

pub fn get_status() -> Result<bool> {
    let status: Status = reqwest::get("http://localhost:4444/status")?.json()?;
    Ok(status.value.ready)
    
//        .and_then(|response| response.json())
//        .map(|status: Status| status.value.ready)
}

#[cfg(test)]
mod test {
    use super::*;
    
    #[test]
    fn new_session() {
        assert_eq!(get_status().unwrap(), true);

        let session = Session::new();
        match session {
            Ok(session) => {
                assert!(session.id.len() > 0);
                session.delete();
            },
            Err(e) => assert!(false, "did not create session {}", e)
        }
    }
}
