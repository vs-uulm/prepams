const sqlite3 = require('sqlite3');
const { open } = require('sqlite'); 

module.exports = {
  async openDatabase() {
    const db = await open({
      filename: process.env['PREPAMS_DB_PATH'] || './data/prepams.db',
      driver: sqlite3.Database
    });

    await db.migrate();
    return db;
  }
}
