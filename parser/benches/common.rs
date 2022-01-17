use dotenv;

pub fn setup() {
    dotenv::dotenv().expect("Failed to read .env file");
}
