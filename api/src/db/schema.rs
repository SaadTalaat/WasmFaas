// @generated automatically by Diesel CLI.

diesel::table! {
    functions (id) {
        id -> Int4,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        arity -> Int4,
        name -> Varchar,
        uri -> Varchar,
        user_uri -> Varchar,
        signature -> Jsonb,
    }
}
