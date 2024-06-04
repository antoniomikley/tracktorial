use anyhow::anyhow;
use reqwest::{blocking, header, redirect, StatusCode};
use scraper::{Html, Selector};

/// Representation of an E-Mail address and a password that can be used to login to Factorial
#[derive(Clone)]
pub struct Credential {
    email: String,
    password: String,
}

impl Credential {
    /// Creates a new credential from an E-Mail address and a password.
    pub fn new(email: &str, password: &str) -> Credential {
        Credential {
            email: String::from(email),
            password: String::from(password),
        }
    }

    /// Creates a new credential from an E-Mail address and retrieves the password from the keyring
    /// or sets an empty password should that fail.
    pub fn new_without_password(email: &str) -> Credential {
        let entry = keyring::Entry::new("tracktorial", email);
        let password = match entry {
            Ok(entry) => entry.get_password().unwrap_or(String::from("")),
            Err(_) => String::from(""),
        };
        Credential {
            email: email.to_string(),
            password,
        }
    }

    /// Gets the password associated with the credentials E-Mail address from the keyring.
    ///
    /// # Errors
    /// Returns an error if the entry in the keyring could not be retrieved or created or if there is
    /// no password associated with this application or the E-Mail address.
    pub fn get_password(&self) -> anyhow::Result<String> {
        let entry = keyring::Entry::new("tracktorial", &self.email)?;
        let password = entry.get_password()?;
        Ok(password)
    }

    pub fn reset_password(&mut self) -> anyhow::Result<()> {
        keyring::Entry::new("tracktorial", &self.email)?.delete_password()?;
        self.password = String::from("");
        Ok(())
    }
    /// Promps the user for the password and saves it in the keyring coupled to the
    /// given E-Mail address.
    ///
    /// # Errors
    /// Returns an error if the entry in the keyring could not be retrieved or created or if the
    /// password could not be read from stdin.
    pub fn ask_for_password(&mut self) -> anyhow::Result<()> {
        let entry = keyring::Entry::new("tracktorial", &self.email)?;
        let password = rpassword::prompt_password("Enter password: ")?;
        entry.set_password(&password)?;
        self.password = password;
        Ok(())
    }
    /// Creates a blocking client and logs it in to Factorial via SAML SSO using an E-Mail address and a password.
    ///
    /// # Errors
    /// - Fails if TLS backend cannot be initialized, or the resolver cannot load the system
    /// configuration.
    /// - Fails if there was an error while sending an HTTP request.
    /// - Fails if an HTTP response containes a non UTF-8 encoded body during the login process.
    /// - Fails if an HTTP response has a body of type application/json, but the body is malformed.
    /// - Fails if ther was an error during the login process.
    ///
    /// # Panics
    /// - Panics when called from within an async runtime.
    pub fn authenticate_client(&self) -> anyhow::Result<blocking::Client> {
        let client = blocking::ClientBuilder::new()
            .redirect(redirect::Policy::none())
            .cookie_store(true)
            .build()?;
        // Öffnet Login-Seite. Setzt factorial_session cookie.
        client
            .get("https://api.factorialhr.com/users/sign_in")
            .send()?;
        // Start SAML Login Prozess. Antwort enthält HTML Form mit authenticity_token,
        // welches für das nächste Request benötigt wird.
        let response = client
            .get("https://api.factorialhr.com/saml_login/new?locale=en-us")
            .send()?;
        if response.status() != StatusCode::OK {
            return Err(anyhow!("Should have received 200 OK, but did not."));
        }
        // Extrahiere das authenticity_token aus dem Body des letzten Requests.
        let mut response_body = Html::parse_document(&response.text().expect("hello"));
        let mut selector = Selector::parse(r#"input[name="authenticity_token"]"#)
            .expect("Could not parse CSS selector group.");
        let authenticity_token = match response_body.select(&selector).next() {
            Some(selection) => selection,
            None => return Err(anyhow!("Could not find the authenticity token.")),
        }
        .attr("value")
        .unwrap();
        // HTML Formular für SAML Login.
        let mail_login_form = [
            ("authenticity_token", &authenticity_token),
            ("return_host", &"api.factorial.com"),
            ("email", &self.email.as_str()),
            ("commit", &"Sign+in+with+SAML+SSO"),
        ];
        // Schicke Formular ab.
        let saml_login_response = client
            .post("https://api.factorialhr.com/saml_login?html[class]=form&locale=en-us")
            .form(&mail_login_form)
            .send()?;

        // Wird umgeleitet zu factorial-production.auth... amazoncognito
        if saml_login_response.status() != StatusCode::FOUND {
            return Err(anyhow!("Request should have been redirected, but was not."));
        }
        let mut redirect_url = saml_login_response.headers().get(header::LOCATION).unwrap();
        let mut response = client.get(redirect_url.to_str()?).send()?;

        // Wird erneut umgeleitet zu login.microsoftonline.com. Erstellt SAML Request.
        if response.status() != StatusCode::FOUND {
            return Err(anyhow!("Request should have been redirected, but was not."));
        }
        redirect_url = response.headers().get(header::LOCATION).unwrap();
        let request = client.get(redirect_url.to_str()?).build()?;
        response = client.execute(request.try_clone().unwrap()).unwrap();

        // Response zu SAML Request enthält Daten, die für das nächste Login Formular
        // benötigt werden.
        let mut json =
            Self::extract_json_from_config_variable_in_response_body(&response.text().unwrap())
                .unwrap();
        let mut data: serde_json::Value = serde_json::from_str(&json).unwrap();

        let saml_login_form = [
            ("login", &self.email.as_str()),
            ("loginfmt", &self.email.as_str()),
            ("passwd", &self.password.as_str()),
            ("canary", &data["canary"].as_str().unwrap_or("")),
            ("ctx", &data["sCtx"].as_str().unwrap_or("")),
            ("hgprequestid", &data["sessionId"].as_str().unwrap_or("")),
            ("flowToken", &data["sFT"].as_str().unwrap_or("")),
            ("i19", &"4564"),
            ("i13", &"0"),
            ("type", &"11"),
            ("ps", &"2"),
            ("NewUser", &"1"),
            ("fspost", &"0"),
            ("i21", &"0"),
            ("CookieDisclosure", &"0"),
            ("IsFidoSupported", &"1"),
            ("isSignupPost", &"0"),
            ("Irt", &""),
            ("IrtPartition", &""),
            ("hisRegion", &""),
            ("hisScaleUnit", &""),
            ("psRNGCDefaultType", &""),
            ("psRNGCEntropy", &""),
            ("psRNGCSLK", &""),
            ("PPSX", &""),
        ];

        // Link von einem vorherigen Request wird modifiziert, da dieser eine Client ID
        // beinhaltet.
        let saml_login_link = format!(
            "https://{}{}",
            request.url().host_str().unwrap_or(""),
            request.url().path().replace("saml2", "login")
        );

        // Schicke Login Formular ab. Antwort enthält Daten für das nächste Formular.
        let saml_request_response = client.post(saml_login_link).form(&saml_login_form).send()?;
        if saml_request_response.status() != StatusCode::OK {
            return Err(anyhow!("Should have received 200 OK, but did not."));
        }

        // Nach Login wird gefragt, ob der Benutzer angemeldet bleiben soll. Braucht ein neues
        // Formular.
        json = Self::extract_json_from_config_variable_in_response_body(
            &saml_request_response.text().unwrap(),
        )?;
        data = serde_json::from_str(&json).unwrap();
        let kmsi_form = [
            // kmsi = keep me signed in
            ("loginOtions", &"1"),
            ("canary", &data["canary"].as_str().unwrap_or("")),
            ("ctx", &data["sCtx"].as_str().unwrap_or("")),
            ("hgprequestid", &data["sessionId"].as_str().unwrap_or("")),
            ("flowToken", &data["sFT"].as_str().unwrap_or("")),
            ("i19", &"2456"),
            ("type", &"28"),
            ("DontShowAgain", &"true"),
        ];
        response = client
            .post("https://login.microsoftonline.com/kmsi")
            .form(&kmsi_form)
            .send()?;

        if response.status() != StatusCode::OK {
            return Err(anyhow!("Should have received 200 OK, but did not."));
        }
        // Antwort von login.microsoftonline.com enthält SAML Response für
        // vorheriges SAML Request.
        response_body = Html::parse_document(&response.text().unwrap());
        selector = Selector::parse(r#"input[name="SAMLResponse"#).unwrap();
        let saml_response = match response_body.select(&selector).next() {
            Some(selection) => selection,
            None => return Err(anyhow!("Could not find the SAMLResponse.")),
        }
        .attr("value")
        .unwrap();
        selector = Selector::parse(r#"input[name="RelayState"]"#).unwrap();
        let relay_state = match response_body.select(&selector).next() {
            Some(selection) => selection,
            None => return Err(anyhow!("Could not find the RelayState.")),
        }
        .attr("value")
        .unwrap();
        selector = Selector::parse(r#"form"#).unwrap();
        let new_url = match response_body.select(&selector).next() {
            Some(selection) => selection,
            None => return Err(anyhow!("Could not find the amozoncognito url.")),
        }
        .attr("action")
        .unwrap();
        // Schicke SAML Response zurück zu amazoncognito.
        let saml_form = [("SAMLResponse", saml_response), ("RelayState", relay_state)];
        response = client.post(new_url).form(&saml_form).send()?;
        // Wird weitergeleitet zu api.factorialhr.com. Setzt neuen
        // factorial_session_cookie.
        if response.status() != StatusCode::FOUND {
            return Err(anyhow!("Request should have been redirected, but was not."));
        }
        redirect_url = response.headers().get(header::LOCATION).unwrap();
        client.get(redirect_url.to_str()?).send()?;
        // Return authentifizierten Client.
        Ok(client)
    }

    fn extract_json_from_config_variable_in_response_body(
        response_body: &str,
    ) -> anyhow::Result<String> {
        let mut max = 0;
        let mut long_line = String::new();
        for line in response_body.lines() {
            if line.len() > max {
                max = line.len();
                long_line = line.to_string();
            }
        }
        Ok(String::from(long_line.get(8..long_line.len() - 1).unwrap()))
    }
}
