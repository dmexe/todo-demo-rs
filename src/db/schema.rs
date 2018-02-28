table! {
    todos (id) {
        id -> Bigint,
        text -> Varchar,
        done -> Bool,
        created_at -> Datetime,
        updated_at -> Datetime,
    }
}
