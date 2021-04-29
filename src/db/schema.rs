table! {
    users (id) {
        id -> Uuid,
        username -> Varchar,
        email -> Varchar,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

table! {
    vocab_book_contents (id) {
        id -> Uuid,
        book_id -> Uuid,
        vocab -> Varchar,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

table! {
    vocab_books (id) {
        id -> Uuid,
        name -> Varchar,
        created_by -> Uuid,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

table! {
    vocab_dicts (id) {
        id -> Uuid,
        vocab -> Varchar,
        partofspeech -> Varchar,
        meaning -> Varchar,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

table! {
    vocab_speeches (id) {
        id -> Uuid,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        vocab -> Varchar,
        mp3 -> Bytea,
    }
}

table! {
    vocabs (vocab) {
        vocab -> Varchar,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

joinable!(vocab_book_contents -> vocab_books (book_id));
joinable!(vocab_book_contents -> vocabs (vocab));
joinable!(vocab_books -> users (created_by));
joinable!(vocab_dicts -> vocabs (vocab));
joinable!(vocab_speeches -> vocabs (vocab));

allow_tables_to_appear_in_same_query!(
    users,
    vocab_book_contents,
    vocab_books,
    vocab_dicts,
    vocab_speeches,
    vocabs,
);
