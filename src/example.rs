use actix_web::{get, post, put, delete, web, App, HttpRequest, HttpResponse, HttpServer, Responder, ResponseError};
use actix_web::http::header::ContentType;
use actix_web::http::StatusCode;
use actix_web::body::BoxBody;

use serde::{Serialize, Deserialize};

use std::fmt::Display;
use std::path;
use std::str::FromStr;
use std::sync::Mutex;

use std::fs;

/*
get, post, put, delete - gives access to Actix Web's built-in macros for specifying the method and path that a defined handler should respond to.
web - Actix Web shares application state with all routes and resources within the same scope. Access the state using the web::Data<T> extractor, where T represents the type of the state. Internally, web::Data uses Arc to offer shared ownership.
App - used to create the application's instance and register the request handlers.
HttpRequest, HttpResponse - gives access to the HTTP request and response pairs.
Responder - Actix Web allows you to return any type as an HttpResponse by implementing a Responder trait that converts into a HttpResponse. User-defined types implement this trait so that they can return directly from handlers.
ResponseError - a handler can return a custom error type in a result if the type implements the ResponseError trait.
ContentType - allows you set the Content-Type in the header of an HttpResponse.
StatusCode - contains bindings and methods for handling HTTP status codes used by Actix Web.
BoxBody - a boxed message body type used as an associated type within the Responder trait implementation.
Serialize, Deserialize - Serde provides a derive macro used to generate serialization implementations for structs defined in a program at compile time.
Display - the ResponseError trait, has a trait bound of fmt::Debug + fmt::Display. To implement the ResponseError trait for a user-defined type, the type must also implement the Debug and Display traits.
Mutex - used to control concurrent access by utilizing a locking mechanism on a shared object.
 */

 fn read_json_file(file_path: String) -> serde_json::Value {
    let content = std::fs::read_to_string(&file_path).unwrap();
    serde_json::from_str(&content).unwrap()
 }

 
fn update_json_file(file_path: String, data: String) -> std::io::Result<()> {

    let converted = serde_json::Value::from_str(&data).map_err(|e|
        std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    let file = fs::OpenOptions::new()
        .write(true)
        .open(&file_path)?;

    let mut writer = std::io::BufWriter::new(file);

    serde_json::to_writer(&mut writer, &converted)?;

    Ok(())

 }

 #[derive(Serialize, Deserialize)]
 struct Ticket {
    nickname: String,
    password: String,
 }

impl Responder for Ticket {
    type Body = BoxBody;

    fn respond_to(self, req: &HttpRequest) -> HttpResponse<Self::Body> {
        let res_body = serde_json::to_string(&self).unwrap();

        HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(res_body)
    }
}

#[get("/ticket")]
async fn get_ticket() -> impl Responder {

    let ticket = Ticket {
        nickname: String::from("Siema"),
        password: String::from("Eniu")
    };

    let response = serde_json::to_string(&ticket).unwrap();

    HttpResponse::Ok()
       .content_type(ContentType::json())
       .body(response)
}

// pub trait Example {
//     fn display(&self);
// }

// impl Example for Ticket {
//     fn display(&self) {
//         println!("The nickname is {}", self.nickname);
//     }
// }
#[actix_web::main]
 async fn main2() -> std::io::Result<()>  {
    let user = Ticket {
        nickname: String::from("Adam"),
        password: String::from("qwerty1234"),
    };
    HttpServer::new(move || 
        App::new()
            .service(get_ticket)
        )
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
 }


 fn main() {
    let user = Ticket {
        nickname: "Guj"
    };
    let empty_string = "".to_string();
    
    println("{}", empty_string.len());
 }