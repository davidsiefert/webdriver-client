
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

    pub fn delete(self) -> Result<bool> {
        self.client.delete(format!("http://localhost:4444/session/{}", self.id).as_str()).send()?.text()?;
        // response: {"value": {}}
        Ok(true)
    }

    pub fn navigate_to(self, url: &str) -> Result<Session> {
        let mut body: HashMap<&str, &str> = HashMap::new();
        body.insert("url", url);
        
        let response = self.client.post(format!("http://localhost:4444/session/{}/url", self.id).as_str()).json(&body).send()?.text()?;
        println!("navigate_to: {}", response);
        
        Ok(self)
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
    fn end_to_end() {
        assert_eq!(get_status().unwrap(), true);

        let result = Session::new()
            .and_then(|session| session.navigate_to("https://www.google.com/"))
            .and_then(|session| session.delete());

        match result {
            Ok(deleted) => assert!(deleted, "did not cleanup session"),
            Err(e) => assert!(false, "did not create session {}", e)
        }
    }
}
