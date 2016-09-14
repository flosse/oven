# oven
simple cookie middleware for Iron

```rust

extern crate hyper;
extern crate oven;

use iron::headers::CookiePair;
use oven::prelude::*;

fn initialize_my_webapp_pls() {
  // do some things, make an iron::Chain...
  chain.link(oven::new(SUPER_SECRET_KEYS_THAT_LETS_BE_HONEST_YOULL_PROBABLY_ACCIDENTALLY_PUT_IN_GITHUB));
}

fn handle_some_requests(req: &mut Request) {
  let foocookie = req.get_cookie("foo"); // foo = Option<&CookiePair>
  // clients can't tamper with foo- it's signed when set and verified when loaded.
  // invalid signatures are equivalent to the cookie not existing.
  let mut resp = Response::new();
  resp.set_cookie("foo", "new and interesting value of foo!");
}
```