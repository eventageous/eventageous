#[derive(Debug)]
pub struct OAuthConfig {
    pub client_id: String,
    pub client_secret: String,
    pub auth_url: String,
    pub token_url: String,
    pub redirect_url: String,
}

impl OAuthConfig {
    pub fn new(
        client_id: String,
        client_secret: String,
        auth_url: String,
        token_url: String,
        redirect_url: String,
    ) -> Self {
        OAuthConfig {
            client_id,
            client_secret,
            auth_url,
            token_url,
            redirect_url,
        }
    }
}
