#[derive(Debug, thiserror::Error)]
pub enum ProtocolError {
    #[error("postcard decode: {0}")]
    Decode(#[from] postcard::Error),
    #[error("postcard encode: {0}")]
    Encode(postcard::Error),
    #[error("unsupported protocol version {0}")]
    UnsupportedVersion(u16),
    #[error("expected ClientHello first, got {0:?}")]
    ExpectedHello(Box<str>),
    #[error("session not authenticated")]
    NotAuthenticated,
}

impl ProtocolError {
    pub fn encode(e: postcard::Error) -> Self {
        ProtocolError::Encode(e)
    }
}
