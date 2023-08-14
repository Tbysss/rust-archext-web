#[macro_use]
extern crate rocket;

use rocket::Config;
use rocket::form::{Context, Contextual, Form};
use rocket::fs::{relative, FileServer, TempFile};
use rocket::http::{ContentType, Status};

use rocket_dyn_templates::Template;

#[derive(Debug, FromForm)]
#[allow(dead_code)]
struct Submission<'v> {
    #[field(validate = len(1..))]
    project: &'v str,
    #[field(validate = ext(ContentType::ZIP))]
    file: TempFile<'v>,
}

#[derive(Debug, FromForm)]
#[allow(dead_code)]
struct Submit<'v> {
    submission: Submission<'v>
}

#[get("/")]
fn index() -> Template {
    Template::render("index", &Context::default())
}

#[post("/", data = "<form>")]
fn submit<'r>(form: Form<Contextual<'r, Submit<'r>>>, config: &Config) -> (Status, Template) {
    let template = match form.value {
        Some(ref submission) => {
            let file_name = submission.submission.file.name().unwrap();
            let extension = submission
                .submission
                .file
                .content_type()
                .and_then(|f| f.extension())
                .unwrap()
                .as_str();
            let file_name_with_extension = vec![file_name, extension].join(".");
            let file_path: &std::path::Path = submission.submission.file.path().unwrap();

            let mut p = config.temp_dir.clone().relative();
            // already exists -> simply show success
            if std::path::Path::new(&p.join(submission.submission.project)).is_dir(){
                Template::render("success", &form.context)
            } else {
                println!("submission: {:#?}", submission);
                p.push(submission.submission.project);
                std::fs::create_dir_all(p.clone()).expect("upload dir created");
                p.push(file_name_with_extension);
                println!(
                    "persiting uploaded data file {:?} to {:?}",
                    file_name, p
                );
                std::fs::rename(file_path, p).expect("failed to persist upload data");
                Template::render("success", &form.context)
            }
        }
        None => {
            Template::render("index", &form.context)
        }
    };

    (form.context.status(), template)
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index, submit])
        .attach(Template::fairing())
        .mount("/", FileServer::from(relative!("/static")))
}
