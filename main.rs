#![feature(proc_macro_hygiene, decl_macro)]

use base64::{decode};
use rocket::{Route};
use rocket::http::{Status, Method::*};
use rocket::handler::{Outcome};
use std::char;
use rocket::Data;



fn check_headers(req: &rocket::Request) -> bool {
  let key = "user:pass";

  let cur_header = req.headers().contains("Proxy-Authorization");
  if !cur_header {
    return false;
  }

  let header_val = req.headers().get_one("Proxy-Authorization");

  if !header_val.is_some() {
    return false;
  }

  let decoded = decode(&header_val.unwrap());

  let decoded = match decoded {
    Ok(d) => d,
    Err(_err) => return false
  };

  let keylen = key.chars().count();
  for (i, n) in decoded.iter().enumerate() {
    
    if i >= keylen || *n as char != key.chars().nth(i).unwrap() {
      return false;
    }
    
  }
  return true;
}

fn forward_req(req: &rocket::Request) -> Result<(), ()> {
  let _client = reqwest::Client::new();

  let host = req.headers().get_one("Host");
  if host == None {
    return Err(());
  } 

  let query;
  if req.uri().query() == None {
    query = "";
  } else {
    query = req.uri().query().unwrap();
  }

  let uri = format!("https://{}{}?{}", host.unwrap(), req.uri().path(), query);
  
  let method = reqwest::Method::from_bytes(req.method().as_str().as_bytes());

  let method = match method {
    Ok(m)  => m,
    Err(_e) => return Err(()),
  };

  let parsed_uri = reqwest::Url::parse(uri.as_str());

  let parsed_uri = match parsed_uri {
    Ok(p) => p,
    Err(_e) => return Err(()),
  };

  let mut request = reqwest::Request::new(method, parsed_uri);

  /* headers & cookies */
  let header_map = reqwest::Request::headers_mut(&mut request);

  for header in req.headers().iter() {
    let name = reqwest::header::HeaderName::from_bytes(header.name().as_bytes());

    let name = match name {
      Ok(n) => n,
      Err(_e) => return Err(()),
    };

    let val = reqwest::header::HeaderValue::from_bytes(header.value().as_bytes());

    let val = match val {
      Ok(v) => v,
      Err(_e) => return Err(()),
    };
    

    header_map.insert(name, val);
  };

  /* body */

  return Ok(())  
}
 
fn handle_req<'r>(req: &'r rocket::Request, _: Data) -> Outcome<'r> {
  let valid_auth = check_headers(req);
  if !valid_auth {
    return Outcome::failure(Status::Unauthorized);
  }
  forward_req(req);
  return Outcome::from(req, "Hello!");
}

fn main() {
  let mut all_routes = vec![];
  for method in &[Get, Put, Post, Delete, Options, Head, Trace, Connect, Patch] {
      all_routes.push(Route::new(*method, "/", handle_req));
      all_routes.push(Route::new(*method, "/<path..>", handle_req));
  }


  rocket::ignite().mount("/", all_routes).launch();
}