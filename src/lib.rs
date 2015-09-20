extern crate cookie;
extern crate iron;
extern crate plugin;

use std::collections::HashMap;
use std::sync::Arc;
use iron::prelude::*;

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
    pub use ::ResponseExt;
    pub use ::RequestExt;
}

pub trait ResponseExt {
    /// Extension method to simplify setting cookies.
    fn set_cookie(&mut self, cookie: cookie::Cookie) -> Result<(), Error>;
}

impl ResponseExt for Response {
    fn set_cookie(&mut self, cookie: cookie::Cookie) -> Result<(), Error> {
        // FIXME: what if there's already a cookie by this name?

        try!(self.get_mut::<ResponseCookies>()).insert(cookie.name.clone(), cookie);

        Ok(())
    }
}

pub trait RequestExt {
    /// Extension method to simplify getting cookies.
    fn get_cookie<'c, 'd>(&'c  mut self, name: &'d str) -> Result<Option<&'c cookie::Cookie>, Error>;
}


impl<'a, 'b> RequestExt for Request<'a, 'b> { 
    fn get_cookie<'c, 'd>(&'c  mut self, name: &'d str) -> Result<Option<&'c cookie::Cookie>, Error> {
        Ok(try!(self.get_mut::<RequestCookies>()).get(name))
    }
}
impl<'a, 'b> plugin::Plugin<Request<'a, 'b>> for RequestCookies {
    type Error = Error;

    fn eval(req: &mut Request) -> Result<HashMap<String, cookie::Cookie>, Error> {
        // try! doesn't work here for some reason
        let signing_key = match req.extensions.get::<SigningKey>() {
            Some(key) => key.clone(),
            None => return Err(Error::NoSigningKey),
        };

        let jar = match req.headers.get::<iron::headers::Cookie>() {
            Some(cookies) => cookies.to_cookie_jar(&signing_key),
            None => cookie::CookieJar::new(&signing_key),
        };

        Ok(jar.signed().iter().map(|c| (c.name.clone(), c)).collect())
    }
}

// Is this a reasonable thing to be doing?
impl plugin::Plugin<Response> for ResponseCookies {
    type Error = Error;

    fn eval(_res: &mut Response) -> Result<HashMap<String, cookie::Cookie>, Error> {
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
    type Value = HashMap<String, cookie::Cookie>;
}

pub struct ResponseCookies;
impl iron::typemap::Key for ResponseCookies {
    type Value = HashMap<String, cookie::Cookie>;
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

        let cookiejar = cookie::CookieJar::new(&self.signing_key);
        if let Some(cookies) = res.extensions.get::<ResponseCookies>() {
            for v in cookies.values().cloned() {
                cookiejar.signed().add(v);
            }

            res.headers.set(iron::headers::SetCookie(cookiejar.delta()));
        } else {
        }
        Ok(res)
    }
}
