use std::str::FromStr;

use anyhow::Result;
use tonic::{
    metadata::{Ascii, MetadataValue},
    service::Interceptor,
};

#[derive(Clone)]
pub struct Auth {
    token: MetadataValue<Ascii>,
}
impl Auth {
    pub(crate) fn new(token: String) -> Result<Self> {
        let token = MetadataValue::from_str(&token)?;
        Ok(Self { token })
    }
}
impl Interceptor for Auth {
    fn call(
        &mut self,
        mut request: tonic::Request<()>,
    ) -> std::result::Result<tonic::Request<()>, tonic::Status> {
        request
            .metadata_mut()
            .insert("authorization", self.token.clone());
        Ok(request)
    }
}
