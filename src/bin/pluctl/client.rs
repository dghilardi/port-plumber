use hyper::{Body, Uri};
use hyper::body::{Bytes, HttpBody};
use hyper::client::connect::Connect;
use std::error::Error as StdError;
use anyhow::bail;
use serde::de::DeserializeOwned;

pub struct SimpleRest<C, B = Body> {
    client: hyper::Client<C, B>
}

impl <C, B> From<hyper::Client<C, B>> for SimpleRest<C, B> {
    fn from(value: hyper::Client<C, B>) -> Self {
        Self {
            client: value,
        }
    }
}

impl <C, B> SimpleRest<C, B>
    where
        C: Connect + Clone + Send + Sync + 'static,
        B: HttpBody + Send + 'static,
        B::Data: Send,
        B::Error: Into<Box<dyn StdError + Send + Sync>>,
{
    pub async fn get<U, Res>(&self, url: U) -> anyhow::Result<Res>
    where
        B: Default,
        U: Into<Uri>,
        Res: DeserializeOwned,
    {
        let res = self.client.get(url.into()).await?;
        if !res.status().is_success() {
            bail!("Response error")
        }
        let res_body: Bytes = hyper::body::to_bytes(res.into_body()).await?;
        let parsed_res = serde_json::from_slice(&res_body[..])?;
        Ok(parsed_res)
    }
}