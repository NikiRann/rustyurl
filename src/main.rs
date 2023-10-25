use mongodb::{bson::doc, options::{ClientOptions, ServerApi, ServerApiVersion}, sync::Client};
use rocket::{form::Form, form::FromForm, get, post, launch, routes, response::{Redirect, content, status}, State, http::Status};
use std::sync::Mutex;
use rand::{distributions::Alphanumeric, Rng};

use serde::{Deserialize, Serialize};

struct DbHandle {
    database: Mutex<mongodb::sync::Database>
}

#[derive(FromForm)]
struct URL {
    url: String
}

#[derive(Debug, Serialize, Deserialize)]
struct URLEntry {
    source: String,
    destination: String,
    timestamp: mongodb::bson::DateTime,
    metadata: mongodb::bson::Document
}

#[get("/small/<url>")]
async fn shortened(db: &State<DbHandle>, url: &str) -> Redirect {
    let data = db.database.lock().unwrap();
    let collection: mongodb::sync::Collection<URLEntry> = data.collection::<URLEntry>("rustyurls");
    println!("source: {}", collection.name());
    let cursor = collection.find_one(doc! { "source": url.to_string() }, None);
    match cursor {
        Ok(x) => {
            let mut dest: String = String::from("");
            for url_entry in x {
                dest = url_entry.destination;
            }
            Redirect::to(dest)
        },
        Err(_x) => {
            Redirect::to("https://mongodb.github.io/")
        }
    }
}

#[post("/create", data = "<dest>")]
fn new(db: &State<DbHandle>, dest: Form<URL>) -> status::Custom<content::RawJson<String>> {
    let shortened: String = rand::thread_rng()
                                .sample_iter(&Alphanumeric)
                                .take(5)
                                .map(char::from)
                                .collect();

    let url_entries = vec![
            URLEntry {
                source: shortened.to_string(),
                destination:  dest.url.to_string(),
                timestamp: mongodb::bson::DateTime::now(),
                metadata: doc! {},
            },
    ];

    let data = db.database.lock().unwrap();
    let collection: mongodb::sync::Collection<URLEntry> = data.collection::<URLEntry>("rustyurls");

    let res = collection.insert_many(url_entries, None);
    match  res {
        Ok(_x) => println!("Okay!"),
        Err(x) => println!("{}", x)
    }
    let formatted_json = format!("{{ \"shortened\": \"small/{}\" }}", shortened);
    let json = content::RawJson(formatted_json);
    status::Custom(Status::ImATeapot, json)
}

#[launch]
fn rocket() -> _ {
    rocket::build().manage(DbHandle { database: Into::into(db_connect()) }).mount("/", routes![shortened]).mount("/", routes![new])
}

fn db_connect() -> mongodb::sync::Database {
    let mut client_options =
    ClientOptions::parse("mongodb+srv://admin:admin@cluster0.2otdo5j.mongodb.net/?retryWrites=true&w=majority").ok().unwrap();
    // Set the server_api field of the client_options object to Stable API version 1
    let server_api = ServerApi::builder().version(ServerApiVersion::V1).build();
    client_options.server_api = Some(server_api);
    // Get a handle to the cluster
    let client = Client::with_options(client_options).ok().unwrap();
    // Ping the server to see if you can connect to the cluster
    // Get a handle to a database.
    let db = client.database("urls");

    return db;
}