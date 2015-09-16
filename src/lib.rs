extern crate cookie;
extern crate iron;

use iron::prelude::*;

pub struct OvenBefore {
    signing_key: Vec<u8>,
}
pub struct OvenAfter;

pub fn create(signing_key: Vec<u8>) -> (OvenBefore, OvenAfter) {
    (OvenBefore { signing_key: signing_key }, OvenAfter)
}

/// Call when you create a Response, to deal with cookies
pub fn init_response(resp: &mut Response, signing_key: &[u8]) {
    resp.extensions.insert::<ResponseCookieJar>(cookie::CookieJar::new(signing_key));
}

pub struct RequestCookieJar;
impl iron::typemap::Key for RequestCookieJar {
    type Value = cookie::CookieJar<'static>;
}

pub struct ResponseCookieJar;
impl iron::typemap::Key for ResponseCookieJar {
    type Value = cookie::CookieJar<'static>;
}

impl iron::BeforeMiddleware for OvenBefore {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        req.extensions
           .insert::<RequestCookieJar>(match req.headers.get::<iron::headers::Cookie>() {
               Some(cookies) => {
                   cookies.to_cookie_jar(&self.signing_key)
               }
               None => {
                   cookie::CookieJar::new(&self.signing_key)
               }
           });

        Ok(())
    }
}


impl iron::AfterMiddleware for OvenAfter {
    fn after(&self, _: &mut Request, mut res: Response) -> IronResult<Response> {
        if let Some(cookiejar) = res.extensions.get::<ResponseCookieJar>() {
            res.headers.set(iron::headers::SetCookie(cookiejar.delta()));
        } else {
            // shouldn't be any other Set-Cookie headers
            debug_assert!(!res.headers.has::<iron::headers::SetCookie>());
        }
        Ok(res)
    }
}
