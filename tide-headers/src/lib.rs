//! Crate that provides helpers, and/or middlewares for tide
//! related to http headers.
#![feature(async_await)]
#![warn(
    nonstandard_style,
    rust_2018_idioms,
    future_incompatible,
    missing_debug_implementations
)]

use futures::future::BoxFuture;
use futures::prelude::*;
use log::trace;

use http::{
    header::{HeaderValue, IntoHeaderName},
    HeaderMap, HttpTryFrom,
};

use tide_core::{
    middleware::{Middleware, Next},
    Context, Response,
};

/// Middleware for providing a set of default headers for all responses.
#[derive(Clone, Default, Debug)]
pub struct DefaultHeaders {
    headers: HeaderMap,
}

impl DefaultHeaders {
    /// Construct a new instance with an empty list of headers.
    pub fn new() -> DefaultHeaders {
        Self::default()
    }

    /// Add a header to the default header list.
    pub fn header<K, V>(mut self, key: K, value: V) -> Self
    where
        K: IntoHeaderName,
        HeaderValue: HttpTryFrom<V>,
    {
        let value = HeaderValue::try_from(value)
            .map_err(Into::into)
            .expect("Cannot create default header");

        self.headers.append(key, value);
        self
    }
}

impl<Data: Send + Sync + 'static> Middleware<Data> for DefaultHeaders {
    fn handle<'a>(&'a self, cx: Context<Data>, next: Next<'a, Data>) -> BoxFuture<'a, Response> {
        FutureExt::boxed(async move {
            let mut res = next.run(cx).await;
            let headers = res.headers_mut();
            for (key, value) in self.headers.iter() {
                trace!("add default: {} {:?}", &key, &value);
                headers.entry(key).unwrap().or_insert_with(|| value.clone());
            }
            res
        })
    }
}
