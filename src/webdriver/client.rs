
use reqwest::Client;
use reqwest::get;

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
    client: Client,
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

#[derive(Deserialize)]
struct StringValue {
    value: String,
}

impl Drop for Session {
    fn drop(&mut self) {
        let url = format!("http://localhost:4444/session/{}", self.id);
        let _ = self.client.delete(url.as_str()).send();
        // response: {"value": {}}
    }
}

impl Session {
    pub fn run<T>(program: fn(&Session) -> Result<T>) -> Result<T> {
        Session::new()
            .and_then(|session| {
                program(&session)
            })
    }
    
    pub fn new() -> Result<Session> {
        let client = Client::new();
        let body: HashMap<String, String> = HashMap::new();
        
        let response: NewSession = client.post("http://localhost:4444/session").json(&body).send()?.json()?;
        let session = Session{
            client: client,
            id: response.value.session_id,
        };

        Ok(session)
    }

    pub fn navigate_to(&self, url: &str) -> Result<bool> {
        let mut body: HashMap<&str, &str> = HashMap::new();
        body.insert("url", url);
        
        self.client.post(format!("http://localhost:4444/session/{}/url", self.id).as_str()).json(&body).send()?.text()?;
        // response: {"value": {}}
        
        Ok(true)
    }

    pub fn get_title(&self) -> Result<String> {
        let response: StringValue = self.client.get(format!("http://localhost:4444/session/{}/title", self.id).as_str()).send()?.json()?;
        Ok(response.value)
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
    let status: Status = get("http://localhost:4444/status")?.json()?;
    Ok(status.value.ready)
}

#[cfg(test)]
mod test {
    use super::*;
    
    #[test]
    fn run_closure_end_to_end_test() {
        let status = get_status()
            .expect("webdriver server status check");
        assert!(status, "webdriver not ready");

        let result = Session::run(|session| {
            session.navigate_to("https://www.google.com")?;
            let title = session.get_title()?;
            if title != "Google" {
                Err(format!("incorrect page title: {}", title).as_str().into())
            } else {
                Ok(())
            }
        });

        assert!(result.is_ok(), "browsing failed {}", result.err().unwrap());
    }
}
