pub struct DbConnection;

impl DbConnection {
    pub fn execute(&self, query: &str) {
        println!("Executing: {}", query);
    }
}

pub struct ItemRepository {
    db: DbConnection,
}

impl ItemRepository {
    pub fn new() -> Self {
        Self { db: DbConnection }
    }
    
    pub fn get_item(&self, id: &str) -> String {
        self.db.execute(&format!("SELECT * FROM items WHERE id = '{}'", id));
        "item_data".to_string()
    }
}
