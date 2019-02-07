#![feature(proc_macro_hygiene, decl_macro, toowned_clone_into)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate serde_derive;
#[macro_use(bson, doc)] extern crate bson;
extern crate mongodb;
extern crate serde;
extern crate rocket_contrib;
extern crate tera;

use bson::{Document, Bson};
use bson::oid::ObjectId;
use mongodb::{Client, ThreadedClient};
use mongodb::db::{Database, ThreadedDatabase};
use mongodb::coll::Collection;
use serde::Serialize;
use serde::de::DeserializeOwned;
use rocket_contrib::templates::Template;
use tera::Context;

#[derive(Serialize, Deserialize, Debug)]
struct UserDocument {
    pub _id: ObjectId,
    pub name: String,
    pub age: usize
}

static mut DB: Option<Database> = None;

#[get("/")]
fn index() -> Template {
    let mut context = Context::new();

    context.add("my_message", &"This is a test message");
    Template::render("layout", &context)
}

#[get("/<name>")]
fn get_info(name: String) -> String {
    let db: Database;
    unsafe {
        db = DB.clone().unwrap();
    }

    let collection: Collection = db.collection("users");
    let user = find_by_name(&collection, name);
    format!("{} is {} years old!", user.name, user.age)
}

fn find_by_name(coll: &Collection, name: String) -> UserDocument {
    let user = coll.find_one(Some(doc!{ "name": name }), None).ok().expect("Failed to find_one").unwrap();
    to_struct(user)
}

fn to_doc<T: Serialize>(item: &T) -> ::bson::Document {
    let bson = ::bson::to_bson(item).ok().expect("Couldn't serialize to doc");
    match bson {
        Bson::Document(d) => d,
        _ => panic!("Couldn't serialize to doc")
    }
}

fn to_struct<T: DeserializeOwned>(doc: Document) -> T {
    ::bson::from_bson(Bson::Document(doc.clone())).ok().expect("Couldn't deserialize to struct")
}

fn main() {
    let client: Client = Client::connect("localhost", 27017).expect("Failed to initialize standalone client.");
    unsafe {
        DB = Some(client.db("rusttest"));
    }

    rocket::ignite()
        .mount("/", routes![index])
        .mount("/info", routes![get_info])
        .attach(Template::fairing())
        .launch();
}
