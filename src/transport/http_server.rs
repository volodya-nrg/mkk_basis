mod handlers;

use handlers::Handlers;

pub struct HTTPServer {}

impl HTTPServer {
    pub fn new() -> Self {
        println!("create new HTTPServer");
        let handlers: Handlers = Handlers::new();
        Self {}
    }
}
