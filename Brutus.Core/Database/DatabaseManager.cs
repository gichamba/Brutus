using Microsoft.Data.Sqlite;

namespace Brutus.Core.Database
{
    /// <summary>
    /// Manages the creation and connection to the SQLite database.
    /// </summary>
    /// <param name="databasePath">The path to the SQLite database file. Defaults to "progress.db".</param>
    public class DatabaseManager(string databasePath = "Brutus.db")
    {
        /// <summary>
        /// Initializes the database by creating the necessary tables if they do not already exist.
        /// If the database file already exists, this method does nothing.
        /// </summary>
        public void InitializeDatabase()
        {
            // If the database file already exists, we assume the schema is correctly set up.
            if (File.Exists(databasePath))
            {
                return;
            }

            using SqliteConnection? connection = new($"Data Source={databasePath}");
            connection.Open();

            // SQL command to create the table for tracking individual PDF files.
            string? createFileListTable = """
                                          
                                                              CREATE TABLE file_list (
                                                                  id INTEGER PRIMARY KEY AUTOINCREMENT,
                                                                  file_path TEXT NOT NULL UNIQUE,
                                                                  file_hash TEXT NOT NULL,
                                                                  status TEXT NOT NULL CHECK(status IN ('PENDING', 'IN_PROGRESS', 'COMPLETED', 'FAILED')),
                                                                  has_password INTEGER NOT NULL DEFAULT 0,  -- 0=unknown, 1=yes, 2=no
                                                                  password_found TEXT DEFAULT NULL,
                                                                  started_at DATETIME DEFAULT NULL,
                                                                  completed_at DATETIME DEFAULT NULL,
                                                                  total_time_minutes REAL DEFAULT NULL,
                                                                  instance_id TEXT DEFAULT NULL
                                                              );
                                                          
                                          """;

            // SQL command to create the table for tracking password batches.
            string? createTrialBatchesTable = """
                                              
                                                                  CREATE TABLE trial_batches (
                                                                      id INTEGER PRIMARY KEY AUTOINCREMENT,
                                                                      file_id INTEGER NOT NULL,
                                                                      batch_index INTEGER NOT NULL,
                                                                      range_from TEXT NOT NULL,
                                                                      range_to TEXT NOT NULL,
                                                                      status TEXT NOT NULL CHECK(status IN ('PENDING', 'CHECKED_OUT', 'COMPLETED')),
                                                                      checked_out_at DATETIME DEFAULT NULL,
                                                                      completed_at DATETIME DEFAULT NULL,
                                                                      instance_id TEXT DEFAULT NULL,
                                                                      FOREIGN KEY (file_id) REFERENCES file_list(id),
                                                                      UNIQUE(file_id, batch_index)
                                                                  );
                                                              
                                              """;

            SqliteCommand? command = connection.CreateCommand();
            command.CommandText = createFileListTable;
            command.ExecuteNonQuery();

            command.CommandText = createTrialBatchesTable;
            command.ExecuteNonQuery();
        }

        /// <summary>
        /// Gets the connection string for the SQLite database.
        /// </summary>
        /// <returns>The database connection string.</returns>
        public string GetConnectionString()
        {
            return $"Data Source={databasePath}";
        }
    }
}