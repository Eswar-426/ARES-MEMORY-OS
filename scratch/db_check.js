const fs = require('fs');
const sqlite3 = require('sqlite3');
const db = new sqlite3.Database('benchmark.db', (err) => {
    if (err) {
        console.error("Failed to open DB:", err.message);
        process.exit(1);
    }
});

db.all("SELECT * FROM telemetry_reports", [], (err, rows) => {
    if (err) {
        console.error("Query failed:", err.message);
    } else {
        console.log(`Found ${rows.length} rows.`);
        console.log(rows);
    }
});
