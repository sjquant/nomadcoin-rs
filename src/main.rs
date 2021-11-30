#[macro_use]
extern crate rocket;
use nomadcoin::{Block, BlockChain};
use rocket::serde::Serialize;
use rocket::State;
use rocket_dyn_templates::Template;
use std::{collections::HashMap, sync::Mutex};

#[derive(Serialize)]
struct HomeTemplateContext<'r> {
    page_title: &'r str,
    blocks: &'r Vec<Block>,
}

#[get("/")]
fn home(chain_state: &State<Mutex<BlockChain>>) -> Template {
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

#[get("/add")]
fn add() -> Template {
    let mut context: HashMap<&str, &str> = HashMap::new();
    context.insert("page_title", "Add");
    Template::render("add", &context)
}

#[launch]
fn rocket() -> _ {
    let mut chain = BlockChain::new();
    chain.add_block("HI");
    chain.add_block("Bye");

    let chain_mutex = Mutex::new(chain);
    rocket::build()
        .mount("/", routes![home, add])
        .attach(Template::fairing())
        .manage(chain_mutex)
}
