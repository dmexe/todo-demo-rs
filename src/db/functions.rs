use diesel;

/// Creates a LAST_INSERT_ID mysql function to use in insert statements.
no_arg_sql_function!(last_insert_id, diesel::sql_types::Bigint);
