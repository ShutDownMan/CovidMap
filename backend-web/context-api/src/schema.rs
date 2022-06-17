table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector};
    use pgvector::{Vector};

    document (id) {
        id -> Int4,
        title -> Text,
        paper_id -> Text,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector};
    use pgvector::{Vector};

    document_text (id) {
        id -> Int4,
        text -> Text,
        tsv -> Nullable<Tsvector>,
        id_text_type -> Nullable<Int4>,
        id_document -> Nullable<Int4>,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector};
    use pgvector::{Vector};

    embedding (id) {
        id -> Int4,
        value -> Vector,
        id_model -> Nullable<Int4>,
        id_document_text -> Nullable<Int4>,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector};
    use pgvector::{Vector};

    model (id) {
        id -> Int4,
        name -> Nullable<Text>,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector};
    use pgvector::{Vector};

    text_type (id) {
        id -> Int4,
        description -> Text,
    }
}

joinable!(document_text -> document (id_document));
joinable!(document_text -> text_type (id_text_type));
joinable!(embedding -> document_text (id_document_text));
joinable!(embedding -> model (id_model));

allow_tables_to_appear_in_same_query!(
    document,
    document_text,
    embedding,
    model,
    text_type,
);
