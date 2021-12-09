#[macro_use]
extern crate rocket;
use nomadcoin::{Block, BlockChain};
use rocket::form::{Form, FromForm};
use rocket::response::{Flash, Redirect};
use rocket::serde::Serialize;
use rocket::State;
use rocket_dyn_templates::Template;
use std::{collections::HashMap, sync::Mutex};

#[derive(Serialize)]
struct HomeTemplateContext<'r> {
    page_title: &'r str,
    blocks: &'r Vec<Block>,
}

#[derive(FromForm)]
struct BlockForm {
    block_data: String,
}

#[get("/")]
fn page_home(chain_state: &State<Mutex<BlockChain>>) -> Template {
    let chain = chain_state.lock().unwrap();
    let blocks = chain.all_blocks();
    Template::render(
        "home",
        &HomeTemplateContext {
            page_title: "Home",
            blocks: blocks,
        },
    )
}

#[get("/")]
fn page_add() -> Template {
    let mut context: HashMap<&str, &str> = HashMap::new();
    context.insert("page_title", "Add");
    Template::render("add", &context)
}

#[post("/", data = "<block_form>")]
fn add_block(
    block_form: Form<BlockForm>,
    chain_state: &State<Mutex<BlockChain>>,
) -> Flash<Redirect> {
    let block = block_form.into_inner();
    let mut chain = chain_state.lock().unwrap();

    if block.block_data.is_empty() {
        Flash::error(Redirect::to("/"), "Block data cannot be empty.")
    } else {
        chain.add_block(&block.block_data);
        Flash::success(Redirect::to("/"), "Block successfully added.")
    }
}

#[launch]
fn rocket() -> _ {
    let chain = BlockChain::new();
    let chain_mutex = Mutex::new(chain);
    rocket::build()
        .mount("/", routes![page_home])
        .mount("/add", routes![page_add, add_block])
        .attach(Template::fairing())
        .manage(chain_mutex)
}
