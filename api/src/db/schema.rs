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

diesel::table! {
    invoke_requests (id) {
        id -> Int4,
        created_at -> Timestamp,
        function_id -> Int4,
        user_addr -> Varchar,
        payload -> Nullable<Jsonb>,
    }
}

diesel::joinable!(invoke_requests -> functions (function_id));

diesel::allow_tables_to_appear_in_same_query!(
    functions,
    invoke_requests,
);
