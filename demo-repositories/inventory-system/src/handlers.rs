use crate::repository::{DbConnection, ItemRepository};

// Compliant Handler
pub fn get_item_handler(repo: &ItemRepository, id: &str) {
    let data = repo.get_item(id);
    println!("Response: {}", data);
}

// ROGUE HANDLER: Violates ADR-3 by bypassing the repository and directly constructing a DB connection
pub fn quick_stock_update_handler(id: &str, stock: i32) {
    // VIOLATION: Handlers shouldn't directly use DbConnection!
    let db = DbConnection;
    db.execute(&format!("UPDATE items SET stock = {} WHERE id = '{}'", stock, id));
    println!("Stock updated fast!");
}
