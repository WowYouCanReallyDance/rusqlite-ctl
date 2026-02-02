use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "rqctl",
    about = "SQLite CLI tool for DDL, DQL, and DML operations",
    version = "0.1.0"
)]
struct Cli {
    /// Path to the SQLite database file (default: ./data.db)
    #[arg(short, long, default_value = "./data.db")]
    db: String,

    /// Database directory (default: current directory)
    #[arg(short, long)]
    dir: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// DDL operations (CREATE, ALTER, DROP)
    Ddl {
        /// SQL statement to execute
        sql: String,
    },
    
    /// DQL operations (SELECT)
    Dql {
        /// SQL SELECT statement to execute
        sql: String,
    },
    
    /// DML operations (INSERT, UPDATE, DELETE)
    Dml {
        /// SQL statement to execute
        sql: String,
    },
    
    /// Execute arbitrary SQL statement
    Sql {
        /// SQL statement to execute
        sql: String,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Construct database path
    let db_path = if let Some(dir) = cli.dir {
        PathBuf::from(dir).join(&cli.db)
    } else {
        PathBuf::from(&cli.db)
    };

    // Ensure the directory exists
    if let Some(parent) = db_path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }

    let conn = rusqlite::Connection::open(&db_path)?;
    
    match cli.command {
        Commands::Ddl { sql } => {
            execute_ddl(&conn, &sql)?;
        }
        Commands::Dql { sql } => {
            execute_dql(&conn, &sql)?;
        }
        Commands::Dml { sql } => {
            execute_dml(&conn, &sql)?;
        }
        Commands::Sql { sql } => {
            execute_sql(&conn, &sql)?;
        }
    }

    Ok(())
}

/// Execute DDL operations (CREATE, ALTER, DROP)
fn execute_ddl(conn: &rusqlite::Connection, sql: &str) -> anyhow::Result<()> {
    let affected_rows = conn.execute(sql, [])?;
    println!("✓ DDL executed successfully. Rows affected: {}", affected_rows);
    Ok(())
}

/// Execute DQL operations (SELECT)
fn execute_dql(conn: &rusqlite::Connection, sql: &str) -> anyhow::Result<()> {
    let mut stmt = conn.prepare(sql)?;
    let column_count = stmt.column_count();
    
    // Print column names
    let column_names: Vec<String> = (0..column_count)
        .map(|i| stmt.column_name(i).unwrap_or("???").to_string())
        .collect();
    println!("{}", column_names.join(" | "));
    println!("{}", "-".repeat(column_names.join(" | ").len()));
    
    // Print rows
    let rows = stmt.query_map([], |row| {
        let mut values = Vec::new();
        for i in 0..column_count {
            let value: String = match row.get::<_, String>(i) {
                Ok(v) => v,
                Err(_) => match row.get::<_, i32>(i) {
                    Ok(v) => v.to_string(),
                    Err(_) => match row.get::<_, f64>(i) {
                        Ok(v) => v.to_string(),
                        Err(_) => "NULL".to_string(),
                    }
                }
            };
            values.push(value);
        }
        Ok(values)
    })?;

    let mut row_count = 0;
    for row in rows {
        let values = row?;
        println!("{}", values.join(" | "));
        row_count += 1;
    }
    
    println!("\n✓ Query returned {} rows", row_count);
    Ok(())
}

/// Execute DML operations (INSERT, UPDATE, DELETE)
fn execute_dml(conn: &rusqlite::Connection, sql: &str) -> anyhow::Result<()> {
    let affected_rows = conn.execute(sql, [])?;
    println!("✓ DML executed successfully. Rows affected: {}", affected_rows);
    Ok(())
}

/// Execute arbitrary SQL statement
fn execute_sql(conn: &rusqlite::Connection, sql: &str) -> anyhow::Result<()> {
    // Determine if it's a SELECT query
    let trimmed_sql = sql.trim();
    if trimmed_sql.to_uppercase().starts_with("SELECT") {
        return execute_dql(conn, trimmed_sql);
    } else if trimmed_sql.to_uppercase().starts_with("INSERT") 
        || trimmed_sql.to_uppercase().starts_with("UPDATE")
        || trimmed_sql.to_uppercase().starts_with("DELETE") {
        return execute_dml(conn, trimmed_sql);
    } else if trimmed_sql.to_uppercase().starts_with("CREATE")
        || trimmed_sql.to_uppercase().starts_with("ALTER")
        || trimmed_sql.to_uppercase().starts_with("DROP") {
        return execute_ddl(conn, trimmed_sql);
    } else {
        let affected_rows = conn.execute(sql, [])?;
        println!("✓ SQL executed successfully. Rows affected: {}", affected_rows);
    }
    
    Ok(())
}
