extern crate iron;
extern crate plugin;

use std::collections::HashMap;
use std::sync::Arc;
use iron::prelude::*;
use iron::headers::{CookieJar, CookiePair};

pub struct OvenBefore {
    signing_key: Arc<Vec<u8>>,
}

pub struct OvenAfter {
    signing_key: Arc<Vec<u8>>,
}

pub fn new(signing_key: Vec<u8>) -> (OvenBefore, OvenAfter) {
    let arc = Arc::new(signing_key);

    let before = OvenBefore { signing_key: arc.clone() };
    let after = OvenAfter { signing_key: arc };

    (before, after)
}

#[derive(Debug, Copy, Clone)]
pub enum Error {
    NoSigningKey,
}


pub mod prelude {
    pub use ResponseExt;
    pub use RequestExt;
}

pub trait ResponseExt {
    /// Extension method to simplify setting cookies.
    fn set_cookie(&mut self, name: &str, value: &str);
}

impl ResponseExt for Response {
    fn set_cookie(&mut self, name: &str, value: &str) {
        // FIXME: what if there's already a cookie by this name?
        let cookie = CookiePair::new(name.to_owned(), value.to_owned());
        self.get_mut::<ResponseCookies>().unwrap().insert(name.to_string(), cookie);
    }
}

pub trait RequestExt {
    /// Extension method to simplify getting cookies.
    fn get_cookie<'c, 'd>(&'c mut self, name: &'d str) -> Option<&CookiePair>;
}


impl<'a, 'b> RequestExt for Request<'a, 'b> {
    fn get_cookie<'c, 'd>(&'c mut self, name: &'d str) -> Option<&CookiePair> {
        self.get_mut::<RequestCookies>().unwrap().get(name)
    }
}
impl<'a, 'b> plugin::Plugin<Request<'a, 'b>> for RequestCookies {
    type Error = Error;

    fn eval(req: &mut Request) -> Result<HashMap<String, CookiePair>, Error> {
        // try! doesn't work here for some reason
        let signing_key = match req.extensions.get::<SigningKey>() {
            Some(key) => key.clone(),
            None => return Err(Error::NoSigningKey),
        };

        let jar = match req.headers.get::<iron::headers::Cookie>() {
            Some(cookies) => cookies.to_cookie_jar(&signing_key),
            None => iron::headers::CookieJar::new(&signing_key),
        };

        Ok(jar.signed().iter().map(|c| (c.name.clone(), c)).collect())
    }
}

// Is this a reasonable thing to be doing?
impl plugin::Plugin<Response> for ResponseCookies {
    type Error = Error;

    fn eval(_res: &mut Response) -> Result<HashMap<String, CookiePair>, Error> {
        Ok(HashMap::new())
    }
}

// Private type stashed in Request/Response extensions
struct SigningKey;
impl iron::typemap::Key for SigningKey {
    type Value = Arc<Vec<u8>>;
}

pub struct RequestCookies;
impl iron::typemap::Key for RequestCookies {
    type Value = HashMap<String, CookiePair>;
}

pub struct ResponseCookies;
impl iron::typemap::Key for ResponseCookies {
    type Value = HashMap<String, CookiePair>;
}

impl iron::BeforeMiddleware for OvenBefore {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        req.extensions.insert::<SigningKey>(self.signing_key.clone());
        Ok(())
    }
}


impl iron::AfterMiddleware for OvenAfter {
    fn after(&self, _: &mut Request, mut res: Response) -> IronResult<Response> {
        // shouldn't be any other Set-Cookie headers
        debug_assert!(!res.headers.has::<iron::headers::SetCookie>());

        let cookiejar = CookieJar::new(&self.signing_key);
        if let Some(cookies) = res.extensions.get::<ResponseCookies>() {
            for c in cookies.values().cloned() {
                cookiejar.signed().add(c);
            }

            res.headers.set(iron::headers::SetCookie(cookiejar.delta()));
        } else {
        }
        Ok(res)
    }
}
