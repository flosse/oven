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
    signing_key: Arc<Vec<u8>>
}

pub enum Error {
    NoSigningKey
}

impl<'a, 'b> plugin::Plugin<Request<'a, 'b>> for RequestCookies { 
    type Error = Error; 

    fn eval(req: &mut Request) -> Result<HashMap<String, cookie::Cookie>, Error> {
        // try! doesn't work here for some reason
        let signing_key = match req.extensions.get::<SigningKey>() {
            Some(key) => key.clone(),
            None => return Err(Error::NoSigningKey)
        };

        let jar = match req.headers.get::<iron::headers::Cookie>() {
            Some(cookies) => cookies.to_cookie_jar(&signing_key),
            None => cookie::CookieJar::new(&signing_key)
        };

        Ok(jar.signed().iter().map(|c| (c.name.clone(), c)).collect())
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
        req.extensions
           .insert::<SigningKey>(self.signing_key.clone());

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
                cookiejar.add(v);
            }

            res.headers.set(iron::headers::SetCookie(cookiejar.delta()));
        } else {
        }
        Ok(res)
    }
}
