#[macro_use]
extern crate rocket;

use rocket::form::{Context, Contextual, Error, Errors, Form};
use rocket::fs::{relative, FileServer, TempFile};
use rocket::http::{ContentType, Status};
use rocket::Config;
use rocket_dyn_templates::Template;

#[derive(Debug, FromForm)]
#[allow(dead_code)]
struct Submission<'v> {
    #[field(validate = len(1..).or_else(msg!("primary label required")))]
    primary: &'v str,
    secondary: &'v str,
    #[field(validate = accept_content_types())]
    file: TempFile<'v>,
}

#[derive(Debug, FromForm)]
#[allow(dead_code)]
struct Submit<'v> {
    submission: Submission<'v>,
}

fn accept_content_types<'v>(file: &TempFile<'_>) -> Result<(), Errors<'v>> {
    let seven_z = ContentType::new("application", "x-7z-compressed");
    let zip = ContentType::ZIP;
    if let Some(file_ct) = file.content_type() {
        if file_ct == &seven_z || file_ct == &zip {
            return Ok(());
        }
    }

    let msg = match (
        file.content_type().and_then(|c| c.extension()),
        seven_z.extension(),
        zip.extension(),
    ) {
        (Some(a), Some(b), Some(c)) => {
            format!("invalid file type: .{}, must be .{} or .{}", a, b, c)
        }
        (Some(a), None, None) => {
            format!("invalid file type: .{}, must be {} or {}", a, seven_z, zip)
        }
        _ => format!("file type must be .{} or .{}", "7z", "zip"),
    };

    Err(Error::validation(msg))?
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
            // fallback to 7z, since ContentType does not know 7z
            let extension = submission
                .submission
                .file
                .content_type()
                .and_then(|f| f.extension())
                .map(|s| s.as_str())
                .or_else(|| Some("7z"))
                .unwrap();
            let file_name_with_extension = vec![file_name, extension].join(".");
            let file_path: &std::path::Path = submission.submission.file.path().unwrap();

            let mut p = config.temp_dir.clone().relative();
            p.push(submission.submission.primary);
            if !submission.submission.secondary.is_empty() {
                p.push(submission.submission.secondary);
            }
            // already exists -> simply show success
            if !std::path::Path::new(&p.join(&file_name_with_extension)).is_file() {
                println!("submission: {:#?}", submission);
                std::fs::create_dir_all(p.clone()).expect("upload dir created");
                p.push(file_name_with_extension);
                println!("persisting uploaded data file {:?} to {:?}", file_name, p);
                std::fs::rename(file_path, p).expect("failed to persist upload data");
            }
            Template::render("index", &form.context)
        }
        None => Template::render("index", &form.context),
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
