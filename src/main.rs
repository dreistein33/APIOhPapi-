use actix_web::{get, post, put, delete, HttpResponse, HttpServer, Responder, App, web, ResponseError};
use actix_web::http::header::ContentType;

use serde::{Serialize, Deserialize};

use serde_json;

use std::{fs, io::Read, str::FromStr};
use std::io::{BufWriter, Seek};
use std::cell::RefCell;

enum InputError {
    EmptyFieldsError(serde_json::Value),
    InvalidCharacterError(serde_json::Value),
    InvalidLengthError(serde_json::Value),
    UsernameTakenError(serde_json::Value)
}

trait InputErrorMethods {
    fn extract_value(&self) -> &serde_json::Value;
}

impl InputErrorMethods for InputError {
    fn extract_value(&self) -> &serde_json::Value {
        match &self {
            Self::EmptyFieldsError(e) => e,
            Self::InvalidCharacterError(e) => e,
            Self::InvalidLengthError(e) => e,
            Self::UsernameTakenError(e) => e
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct User {
    #[serde(default)]
    username: String,
    #[serde(default)]
    password: String,
}

fn define_error(user: &User) -> Result<(), InputError> {

    if user.username.is_empty() || user.password.is_empty() {
        return Err(InputError::EmptyFieldsError(serde_json::json!({"e": "Cannot pass empty username nor password."})));
    }

    let special_characters: [char; 12] = ['?', ';', '#', '$', '%', '&', '*', '+', '-', '/', ':', '@'];

    if user.username.contains(special_characters) {
        return Err(InputError::InvalidCharacterError(serde_json::json!({"e": "Special characters used in username."})));
    }

    if user.username.len() < 5 || user.password.len() < 5 {
        return Err(InputError::InvalidLengthError(serde_json::json!({"e": "Either password and username must be longer than 5 characters."})));
    }

    if check_if_username_exists(user.username.clone()) {
        return Err(InputError::UsernameTakenError(serde_json::json!({"e": "Username is taken."})));
    }

    Ok(())
}



fn add_user_to_file(users: Vec<User>) -> std::io::Result<()> {
    let file_path = "dzejson.json".to_string();

    let file = fs::File::options()
        .write(true)
        .truncate(true)
        .open(&file_path)?;

    let writer = BufWriter::new(file);

    serde_json::to_writer(writer, &users)?;

    Ok(())

}

fn create_vector_out_of_json_file() -> Vec<User> {
    let file_path = "dzejson.json".to_string();

    let content = fs::read_to_string(&file_path).unwrap();

    let content_to_value = serde_json::Value::from_str(&content).unwrap();

    let mut  user_vector: Vec<User> = Vec::new();

    if let Some(array) = content_to_value.as_array() {
        for credential in array {
            let user = User {
                username: credential["username"].as_str().unwrap().to_owned(),
                password: credential["password"].as_str().unwrap().to_owned()
            };
            user_vector.push(user);
        }
    }

    user_vector
}

fn check_if_username_exists(username: String) -> bool {
    let mut taken = false;

    let file_path = "dzejson.json".to_string();

    let user_info_str = fs::read_to_string(&file_path).unwrap();

    let user_info_json = serde_json::Value::from_str(&user_info_str).unwrap();

    if let Some(array) = user_info_json.as_array() {
        for credential in array {
            if credential["username"] == username {
                taken = true;
                break;
            }
        }
    }

    taken
}

#[get("/users")]
async fn get_user_info() -> impl Responder {
    let file_path = "dzejson.json".to_string();
    let user_info = fs::read_to_string(&file_path).unwrap();

    HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(user_info)
}

#[post("/register")]
async fn register_user(user: web::Json<User>) -> impl Responder{

    let unpacked_user = user.into_inner();

    let mut create_user = false;

    match define_error(&unpacked_user) {
        Ok(_) => create_user = true,
        Err(error_msg) => return HttpResponse::InternalServerError()
            .content_type(ContentType::json())
            .body(error_msg.extract_value().to_string())
    }

    if create_user {
        let mut user_vector = create_vector_out_of_json_file();
        user_vector.push(unpacked_user);
        add_user_to_file(user_vector).unwrap();
    }
  
    HttpResponse::Ok()
        .content_type(ContentType::json())
        .body("Success!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    HttpServer::new(move ||
        App::new()
            .service(get_user_info)
            .service(register_user)
        )
        .bind(("127.0.0.1", 3333))?
        .run()
        .await

}