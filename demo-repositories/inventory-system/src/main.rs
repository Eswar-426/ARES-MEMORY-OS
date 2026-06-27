mod repository;
mod handlers;

fn main() {
    let repo = repository::ItemRepository::new();
    handlers::get_item_handler(&repo, "123");
    handlers::quick_stock_update_handler("123", 50);
}
