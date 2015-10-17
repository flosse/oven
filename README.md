# oven
simple cookie middleware for Iron

```rust

extern crate cookie;
extern crate oven;

use oven::prelude::*;

fn initialize_my_webapp_pls() {
  // do some things, make an iron::Chain...
  chain.link(oven::new("SUPER_SECRET_KEYS_THAT_LETS_BE_HONEST_YOULL_PROBABLY_ACCIDENTALLY_PUT_IN_GITHUB".into()));
}

fn handle_some_requests(req: &mut Request) {
  let foocookie = req.get_cookie("foo".into()); // foo = Option<&cookie::Cookie>
  // clients can't tamper with foo- it's signed when set and verified when loaded.
  // invalid signatures are equivalent to the cookie not existing.
  let mut resp = Response::new();
  resp.set_cookie(Cookie::new("foo".into(), "new and interesting value of foo!".into()));
  
