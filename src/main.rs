#![feature(proc_macro_hygiene, decl_macro, toowned_clone_into)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate serde_derive;
#[macro_use(bson, doc)] extern crate bson;
extern crate mongodb;
extern crate serde;

use bson::{Document, Bson};
use bson::oid::ObjectId;
use mongodb::{Client, ThreadedClient};
use mongodb::db::{Database, ThreadedDatabase};
use mongodb::coll::Collection;
use serde::Serialize;
use serde::de::DeserializeOwned;

#[derive(Serialize, Deserialize, Debug)]
struct UserDocument {
    pub _id: ObjectId,
    pub name: String,
    pub age: usize
}

static mut DB: Option<Database> = None;

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
    to_user(user)
}

fn to_doc<T: Serialize>(item: &T) -> ::bson::Document {
    let bson = ::bson::to_bson(item).ok().expect("Couldn't serialize to doc");
    match bson {
        Bson::Document(d) => d,
        _ => panic!("Couldn't serialize to doc")
    }
}

fn to_user<T: DeserializeOwned>(doc: Document) -> T {
    ::bson::from_bson(Bson::Document(doc.clone())).ok().expect("Couldn't deserialize to struct")
}

fn main() {
    let client: Client = Client::connect("localhost", 27017).expect("Failed to initialize standalone client.");
    unsafe {
        DB = Some(client.db("rusttest"));
    }

    rocket::ignite().mount("/info", routes![get_info]).launch();
}
