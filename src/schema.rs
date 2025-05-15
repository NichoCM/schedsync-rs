// @generated automatically by Diesel CLI.

diesel::table! {
    app_keys (id) {
        id -> Int4,
        app_id -> Int4,
        #[max_length = 255]
        key_preview -> Varchar,
        #[max_length = 255]
        key -> Varchar,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    apps (id) {
        id -> Int4,
        #[max_length = 255]
        client_id -> Varchar,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    groups (id) {
        id -> Int4,
        app_id -> Int4,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    integrations (id) {
        id -> Int4,
        service -> Int2,
        group_id -> Int4,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    oauth2_states (id) {
        id -> Int4,
        state -> Text,
        group_id -> Int4,
        expires_at -> Timestamptz,
        created_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    oauth_integrations (id) {
        id -> Int4,
        service -> Int2,
        integration_id -> Int4,
        expires_at -> Timestamp,
        #[max_length = 255]
        access_token -> Varchar,
        #[max_length = 255]
        refresh_token -> Varchar,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::joinable!(app_keys -> apps (app_id));
diesel::joinable!(groups -> apps (app_id));
diesel::joinable!(integrations -> apps (group_id));
diesel::joinable!(oauth2_states -> groups (group_id));
diesel::joinable!(oauth_integrations -> apps (integration_id));

diesel::allow_tables_to_appear_in_same_query!(
    app_keys,
    apps,
    groups,
    integrations,
    oauth2_states,
    oauth_integrations,
);
