use reqwest::RequestBuilder;

pub trait VersionHeader {
    fn with_version_header(self) -> Self;
}

impl VersionHeader for RequestBuilder {
    fn with_version_header(self) -> Self {
        self.header("X-Version", env!("CARGO_PKG_VERSION"))
    }
}
