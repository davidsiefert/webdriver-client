
use reqwest::Client;
use reqwest::get;
use serde_json::Value;

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

pub struct Element {
    element_id: String,
}

use std::collections::HashMap;

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
        
        // session creation response: {"value": {"sessionId":"9f9c6fdb-ea4a-41fd-84a3-fd54b9b1c58b","capabilities":{"acceptInsecureCerts":false,"browserName":"firefox","browserVersion":"59.0.1","moz:accessibilityChecks":false,"moz:headless":false,"moz:processID":29954,"moz:profile":"/tmp/rust_mozprofile.oH01UKhZUOAL","moz:useNonSpecCompliantPointerOrigin":false,"moz:webdriverClick":true,"pageLoadStrategy":"normal","platformName":"linux","platformVersion":"4.4.0-89-generic","rotatable":false,"timeouts":{"implicit":0,"pageLoad":300000,"script":30000}}}}
        let response: Value = client.post("http://localhost:4444/session").json(&body).send()?.json()?;
        response["value"]["sessionId"].as_str()
            .ok_or(format!("invalid response: {:?}", response).as_str().into())
            .map(|session_id| Session{
              client: client,
              id: session_id.to_string(),
            })
    }

    pub fn navigate_to(&self, url: &str) -> Result<bool> {
        let mut body: HashMap<&str, &str> = HashMap::new();
        body.insert("url", url);

        let path = format!("http://localhost:4444/session/{}/url", self.id);
        self.client.post(path.as_str()).json(&body).send()?.text()?;
        // response: {"value": {}}
        
        Ok(true)
    }

    pub fn get_title(&self) -> Result<String> {
        let response: Value = self.client.get(format!("http://localhost:4444/session/{}/title", self.id).as_str()).send()?.json()?;
        response["value"].as_str()
            .map(|t| t.to_string())
            .ok_or(format!("invalid response: {:?}", response).as_str().into())
    }
    
    pub fn find_element_by_css(&self, selector: &str) -> Result<Element> {
        let mut body: HashMap<&str, &str> = HashMap::new();
        body.insert("using", "css selector");
        body.insert("value", selector);
        
        let path = format!("http://localhost:4444/session/{}/element", self.id);
        let response: Value = self.client.post(path.as_str()).json(&body).send()?.json()?;
        response["value"]
            .as_object()
            .and_then(|m| m.keys().next())
            .map(|element_id| Element{
                element_id: element_id.to_string()
            })
            .ok_or(format!("invalid response: {:?}", response).as_str().into())
    }
}

pub fn get_status() -> Result<bool> {
    let body: Value = get("http://localhost:4444/status")?.json()?;
    body["value"]["ready"].as_bool()
        .ok_or(format!("invalid response from server: {:?}", body).as_str().into())
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
                return Err(format!("incorrect page title: {}", title).as_str().into())
            }
            
            session.find_element_by_css("[name=q]")
        });

        assert!(result.is_ok(), "browsing failed {}", result.err().unwrap());
    }
}
