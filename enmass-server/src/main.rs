#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

#[get("/<name>")]
fn hello_you(name: String) -> String {
    format!("hello, {}", name)
}

#[get("/")]
fn hello_world() -> String {
    "hello, world!".to_string()
}

fn main() {
    rocket::ignite()
        .mount("/hello", routes![hello_world, hello_you])
        .mount("/", routes![hello_world])
        .launch();
}
