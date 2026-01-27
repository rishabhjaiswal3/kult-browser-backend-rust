// why mod
// how can i import all methods from a file
// how can i import a single or multiple methods from a file
mod mongo;

// what is this
// does it change any behaviour of any method
// does it really required
#[tokio::main]
// why put async here
async fn main() {

    // why we are using match here
    // can i have this whole returned value in a variable
    match mongo::connection::connect().await {
        Ok(db) => {
            println!("Database connected: {}", db.name());
        }
        Err(e) => {
            println!("Mongo Error: {}", e);
        }
    }
}
