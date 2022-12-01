const sqlite3 = require('sqlite3');
const { open } = require('sqlite'); 

module.exports = {
  async openDatabase() {
    const db = await open({
      filename: 'prepams.db',
      // filename: ':memory:',
      driver: sqlite3.Database
    });

    await db.migrate();
    return db;
  }
}
