mod paginate;
mod schema;

use rocket::request::FromForm;
use rocket_contrib::databases::{
    diesel,
    diesel::{prelude::*, PgConnection, QueryResult},
};

#[database("postgres")]
pub struct Connection(diesel::PgConnection);

use uuid::Uuid;
use chrono::DateTime;
use chrono::offset::Utc;

use paginate::*;
use schema::*;

#[derive(Queryable, Debug, Identifiable)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Queryable, Debug, Identifiable)]
pub struct VocabBookContent {
    pub id: Uuid,
    pub book_id: Uuid,
    pub vocab: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Queryable, Debug, Identifiable)]
pub struct VocabBook {
    pub id: Uuid,
    pub name: String,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Queryable, Debug, Identifiable)]
pub struct VocabDict {
    pub id: Uuid,
    pub vocab: String,
    pub partofspeech: String,
    pub meaning: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[table_name = "vocab_speeches"]
#[derive(Queryable, Debug, Identifiable)]
pub struct VocabSpeech {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub vocab: String,
    pub mp3: Vec<u8>,
}

#[table_name = "vocabs"]
#[derive(Queryable, Debug, Identifiable)]
#[primary_key(vocab)]
pub struct Vocab {
    pub vocab: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Vocab {
    pub fn get_by_idx(idx: i64, connection: &PgConnection,
    ) -> QueryResult<(Vec<Vocab>, i64)> {
            use std::cmp::max;
            let page_idx = max(idx, 1);
            let paginated_query = vocabs::table.order(vocabs::created_at.desc())
                .paginate(page_idx)
                .per_page(1);
            // println!("{}",diesel::debug_query(&paginated_query));
            paginated_query.load_and_count_pages(connection)
    }
}

impl VocabSpeech {
    pub fn get_by_word(word: &String, connection: &PgConnection) -> QueryResult<VocabSpeech> {
        let query = vocab_speeches::table.filter(vocab_speeches::vocab.eq(word));
        query.get_result::<VocabSpeech>(connection)
    }
}

    // impl Password {
    //     pub fn get_match(
    //         user_id: &Uuid,
    //         passwd: &str,
    //         connection: &PgConnection,
    //     ) -> QueryResult<Password> {
    //         let query = diesel::sql_query(format!(
    //             " SELECT * FROM passwords
    //               WHERE user_id = '{}' AND passwd = crypt('{}', passwd);
    //             ",
    //             user_id, passwd
    //         ));
    //         // println!("{}", diesel::debug_query(&query));
    //         query.get_result::<Password>(connection)
    //     }
    // }

    // impl Manager {
    //     pub fn all(connection: &PgConnection) -> QueryResult<Vec<Manager>> {
    //         managers::table.load::<Manager>(&*connection)
    //     }

    //     pub fn get(id: &Uuid, connection: &PgConnection) -> QueryResult<Manager> {
    //         let query = managers::table.find(id);
    //         // println!("{}", diesel::debug_query(&query));
    //         query.get_result::<Manager>(connection)
    //     }

    //     pub fn get_paginate(
    //         page_idx: i64,
    //         limit: i64,
    //         connection: &PgConnection,
    //     ) -> QueryResult<(Vec<Manager>, i64)> {
    //         use std::cmp::max;
    //         let page_idx = max(page_idx, 1);
    //         let limit = max(limit, 1);
    //         let paginated_query = managers::table
    //             .order(managers::updated_at.desc())
    //             .paginate(page_idx)
    //             .per_page(limit);
    //         // println!("{}",diesel::debug_query(&paginated_query));
    //         paginated_query.load_and_count_pages(connection)
    //     }

    //     pub fn get_paginate_by_review_count(
    //         page_idx: i64,
    //         limit: i64,
    //         connection: &PgConnection,
    //     ) -> QueryResult<(Vec<Manager>, i64)> {
    //         use std::cmp::max;
    //         let page_idx = max(page_idx, 1);
    //         let limit = max(limit, 1);
    //         let query = diesel::sql_query(format!(
    //             " select managers.*
    //               from managers left join reviews
    //               on managers.id = reviews.manager_id
    //               group by managers.id
    //               order by count(reviews.id) desc limit {} offset {};
    //             ",
    //             limit,
    //             (page_idx - 1) * limit
    //         ));
    //         let managers = query.get_results::<Manager>(connection)?;
    //         let total: i64 = managers::table.count().get_result(connection)?;
    //         let total_pages = (total / limit) + ((total % limit > 0) as i64);
    //         Ok((managers, total_pages))
    //     }

    //     pub fn insert(
    //         manager: &InsertableManager,
    //         connection: &PgConnection,
    //     ) -> QueryResult<Manager> {
    //         diesel::insert_into(managers::table)
    //             .values(manager)
    //             .get_result(connection)
    //     }

    //     pub fn update(
    //         id: &Uuid,
    //         manager: &Manager,
    //         connection: &PgConnection,
    //     ) -> QueryResult<Manager> {
    //         diesel::update(managers::table.find(id))
    //             .set(manager)
    //             .get_result(connection)
    //     }

    //     pub fn delete(id: &Uuid, connection: &PgConnection) -> QueryResult<usize> {
    //         diesel::delete(managers::table.find(id)).execute(connection)
    //     }
    // }


    // impl Review {
    //     pub fn all(connection: &PgConnection) -> QueryResult<Vec<Review>> {
    //         reviews::table.load::<Review>(&*connection)
    //     }

    //     pub fn get(id: &Uuid, connection: &PgConnection) -> QueryResult<Review> {
    //         let query = reviews::table.find(id);
    //         // println!("{}", diesel::debug_query(&query));
    //         query.get_result::<Review>(connection)
    //     }

    //     pub fn get_by_manager(
    //         manager: &Manager,
    //         connection: &PgConnection,
    //     ) -> QueryResult<Vec<Review>> {
    //         let query = reviews::table.filter(reviews::manager_id.eq(manager.id));
    //         query.get_results::<Review>(connection)
    //     }

    //     pub fn count_for_manager(manager: &Manager, connection: &PgConnection) -> QueryResult<i64> {
    //         let query = reviews::table
    //             .filter(reviews::manager_id.eq(manager.id))
    //             .count();
    //         query.get_result::<i64>(connection)
    //     }

    //     pub fn insert(review: &InsertableReview, connection: &PgConnection) -> QueryResult<Review> {
    //         diesel::insert_into(reviews::table)
    //             .values(review)
    //             .get_result(connection)
    //     }

    //     pub fn update(
    //         id: &Uuid,
    //         review: &Review,
    //         connection: &PgConnection,
    //     ) -> QueryResult<Review> {
    //         diesel::update(reviews::table.find(id))
    //             .set(review)
    //             .get_result(connection)
    //     }

    //     pub fn delete(id: &Uuid, connection: &PgConnection) -> QueryResult<usize> {
    //         diesel::delete(reviews::table.find(id)).execute(connection)
    //     }
    // }

    // impl User {
    //     pub fn all(connection: &PgConnection) -> QueryResult<Vec<User>> {
    //         users::table.load::<User>(&*connection)
    //     }

    //     pub fn get(id: &Uuid, connection: &PgConnection) -> QueryResult<User> {
    //         let query = users::table.find(id);
    //         // println!("{}", diesel::debug_query(&query));
    //         query.get_result::<User>(connection)
    //     }

    //     pub fn get_by_username(name: &str, connection: &PgConnection) -> QueryResult<User> {
    //         let query = users::table.filter(users::username.eq(name));
    //         // println!("{}", diesel::debug_query(&query));
    //         query.get_result::<User>(connection)
    //     }

    //     pub fn get_by_email(email: &str, connection: &PgConnection) -> QueryResult<User> {
    //         let query = users::table.filter(users::email.eq(email));
    //         // println!("{}", diesel::debug_query(&query));
    //         query.get_result::<User>(connection)
    //     }

    //     pub fn insert_by_email(
    //         email: &str,
    //         passwd: &str,
    //         connection: &PgConnection,
    //     ) -> QueryResult<usize> {
    //         let query = diesel::sql_query(format!(
    //             r#"
    //              WITH user_insert AS (
    //                 INSERT INTO users (username, email, firstname, lastname)
    //                 VALUES ('', '{email}', '', '')
    //                 RETURNING id
    //              )
    //              INSERT INTO passwords (user_id, passwd)
    //                SELECT id, crypt('{password}', gen_salt('bf'))
    //                FROM user_insert;
    //              "#,
    //             email = email,
    //             password = passwd
    //         ));
    //         query.execute(connection)
    //     }

    //     pub fn update(id: &Uuid, user: &User, connection: &PgConnection) -> QueryResult<User> {
    //         diesel::update(users::table.find(id))
    //             .set(user)
    //             .get_result(connection)
    //     }

    //     pub fn delete(id: &Uuid, connection: &PgConnection) -> QueryResult<usize> {
    //         diesel::delete(users::table.find(id)).execute(connection)
    //     }
    // }

