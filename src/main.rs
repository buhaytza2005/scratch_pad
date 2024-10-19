#[allow(unused, dead_code)]
use oauth2::{
    basic::BasicClient,
    reqwest::{async_http_client, http_client},
    AuthUrl, ClientId, ResourceOwnerPassword, ResourceOwnerUsername, Scope, TokenUrl,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = BasicClient::new(
        ClientId::new("client_id".to_string()),
        None,
        AuthUrl::new("https://auth".to_string()).expect("url"),
        Some(TokenUrl::new("https://token".to_string()).expect("Token url")),
    );
    let _token_result = client
        .exchange_password(
            &ResourceOwnerUsername::new("user".to_string()),
            &ResourceOwnerPassword::new("password".to_string()),
        )
        .add_scope(Scope::new("openid".to_string()))
        .request_async(async_http_client)
        .await
        .expect("yes");
    /// standard - uses references and clones them when building
    let mut client = KeycloakClientBuilder::new();
    client.url("https://auth").method("GET");

    let a = client.method("aa").build().expect("build");
    println!("{a:#?}");
    let b = client.method("bb").build().expect("build");
    println!("{b:#?}");

    // fully consumable, cannot use again
    let client_builder = KeycloakClientBuilderConsumer::new()
        .url("http:///")
        .method("GET");

    let client = client_builder.url("https://").build()?;
    println!("{client:#?}");
    //this fails
    //let a = client_builder.build()?;

    // fully consumable, cannot use again
    let client_builder2 = KeycloakClientBuilderConsumer::new()
        .url("http:///")
        .method("GET");
    let client_builder2 = client_builder2.url("https://");

    let client = client_builder2.clone().build()?;
    println!("{client:#?}");

    let client_builder2 = client_builder2.method("http://");
    let client2 = client_builder2.build()?;
    println!("{client2:#?}");

    //TypeState - we should not be able to call build on a builder with no url
    //error[E0599]: no method named `build` found for struct `KeycloakClientBuilderWithTypeState<NoUrl>` in the current scope
    // --> src/main.rs:60:93
    //60  |     let client_builder_with_state = KeycloakClientBuilderWithTypeState::new().method("get").build()?
    //...
    //for multiple states, we cannot build it unless we have set all states.
    //
    //This is pretty cool to determine how methods should be chained and to determine the "MUST"
    //properties when building instead of waiting for compile time

    let client_builder_with_state = KeycloakClientBuilderWithTypeState::new()
        .url("test")
        .method("get")
        .build()?;
    let client_builder_with_multiple_states = KeycloakClientBuilderWithTypestates::new()
        .url("ss")
        .method("gg")
        .build()?;
    Ok(())
}

#[derive(Debug)]
pub struct KeycloakClient {
    pub url: String,
    pub method: String,
}
/// this is for a simple builder. most of the time for internal use is ok.
/// note that all the methods take a reference
#[derive(Default)]
pub struct KeycloakClientBuilder {
    pub url: Option<String>,
    pub method: Option<String>,
}

impl KeycloakClientBuilder {
    pub fn new() -> Self {
        KeycloakClientBuilder::default()
    }

    pub fn url(&mut self, url: impl Into<String>) -> &mut Self {
        self.url = Some(url.into());
        self
    }

    pub fn method(&mut self, method: impl Into<String>) -> &mut Self {
        self.method = Some(method.into());
        self
    }

    pub fn build(&self) -> anyhow::Result<KeycloakClient> {
        let Some(url) = self.url.as_ref() else {
            return Err(anyhow::anyhow!("No URL"));
        };
        let method = self
            .method
            .as_ref()
            .cloned() // could be clone but then we need to do a .to_string() later
            .unwrap_or_else(|| "GET".to_string());
        Ok(KeycloakClient {
            url: url.to_string(),
            method,
        })
    }
}

// this is a consuming pattern, takes all the values from the builder
// we can clone the builder if needed again

#[derive(Default, Clone)]
pub struct KeycloakClientBuilderConsumer {
    url: Option<String>,
    method: Option<String>,
}
impl KeycloakClientBuilderConsumer {
    pub fn new() -> Self {
        KeycloakClientBuilderConsumer::default()
    }

    pub fn url(mut self, url: impl Into<String>) -> Self {
        //this reuses the Option that we have instead of creating a new one
        let _ = self.url.insert(url.into());
        //
        //self.url = Some(url.into());
        self
    }

    pub fn method(mut self, method: impl Into<String>) -> Self {
        let _ = self.method.insert(method.into());
        self
    }

    pub fn build(self) -> anyhow::Result<KeycloakClient> {
        let Some(url) = self.url else {
            return Err(anyhow::anyhow!("No URL"));
        };
        let method = self.method.unwrap_or_else(|| "GET".to_string());
        Ok(KeycloakClient { url, method })
    }
}

///typestates allow certain methods to be called only under certain circumstances
#[derive(Default, Clone)]
pub struct NoUrl;
#[derive(Default, Clone)]
pub struct Url(String);

#[derive(Default, Clone)]
pub struct KeycloakClientBuilderWithTypeState<U> {
    url: U,
    method: Option<String>,
}

/// we move the new() function out as we can pass the NoUrl type and this will now be able to
/// create a default implementation as NoUrl derives default
impl KeycloakClientBuilderWithTypeState<NoUrl> {
    pub fn new() -> Self {
        KeycloakClientBuilderWithTypeState::default()
    }
}

/// just like new, we move the builder outside of the main implementation
/// this allows us to only implement if a url exists
///
impl KeycloakClientBuilderWithTypeState<Url> {
    pub fn build(self) -> anyhow::Result<KeycloakClient> {
        let method = self.method.unwrap_or_else(|| "GET".to_string());
        Ok(KeycloakClient {
            url: self.url.0,
            method,
        })
    }
}
impl<U> KeycloakClientBuilderWithTypeState<U> {
    /// pub fn url(mut self, url: impl Into<String>) -> Self {
    /// we change this from returning self to returning a requestbuilder with url - really cool
    /// we also remove the mut as we don't mutate the builder anymore, we return a new one
    pub fn url(self, url: impl Into<String>) -> KeycloakClientBuilderWithTypeState<Url> {
        //this reuses the Option that we have instead of creating a new one
        KeycloakClientBuilderWithTypeState {
            url: Url(url.into()),
            method: self.method,
        }
    }

    pub fn method(mut self, method: impl Into<String>) -> Self {
        let _ = self.method.insert(method.into());
        self
    }
}
///multiple states
///typestates allow certain methods to be called only under certain circumstances
#[derive(Default, Clone)]
pub struct NoMethod;
#[derive(Default, Clone)]
pub struct Method(String);

///we add the generic M which will be Method or NoMethod
#[derive(Default, Clone)]
pub struct KeycloakClientBuilderWithTypestates<U, M> {
    url: U,
    method: M,
}

/// we move the new() function out as we can pass the NoUrl type and this will now be able to
/// create a default implementation as NoUrl derives default
impl KeycloakClientBuilderWithTypestates<NoUrl, NoMethod> {
    pub fn new() -> Self {
        KeycloakClientBuilderWithTypestates::default()
    }
}

/// just like new, we move the builder outside of the main implementation
/// this allows us to only implement if a url exists
///
impl KeycloakClientBuilderWithTypestates<Url, Method> {
    pub fn build(self) -> anyhow::Result<KeycloakClient> {
        Ok(KeycloakClient {
            url: self.url.0,
            method: self.method.0,
        })
    }
}
impl<U, M> KeycloakClientBuilderWithTypestates<U, M> {
    /// pub fn url(mut self, url: impl Into<String>) -> Self {
    /// we change this from returning self to returning a requestbuilder with url - really cool
    /// we also remove the mut as we don't mutate the builder anymore, we return a new one
    pub fn url(self, url: impl Into<String>) -> KeycloakClientBuilderWithTypestates<Url, M> {
        //this reuses the Option that we have instead of creating a new one
        KeycloakClientBuilderWithTypestates {
            url: Url(url.into()),
            method: self.method,
        }
    }

    pub fn method(
        self,
        method: impl Into<String>,
    ) -> KeycloakClientBuilderWithTypestates<U, Method> {
        KeycloakClientBuilderWithTypestates {
            url: self.url,
            method: Method(method.into()),
        }
    }
}
