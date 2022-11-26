use axum::{
    extract::{Path, Query},
    response::IntoResponse,
    Extension, Json,
};
use futures::{StreamExt, TryStreamExt};
use mongodb::{
    bson::doc,
    options::{AggregateOptions, Collation, FindOptions},
};
use parse::kanjidic::Kanji;
use serde::Deserialize;

mod data;

use crate::{AppError, Database};

pub async fn get_index(db: Extension<Database>) -> Result<Json<Vec<String>>, AppError> {
    let out = db
        .collection::<Kanji>("kanjidic")
        .distinct("literal", None, None)
        .await?;
    Ok(Json(out.iter().map(|b| b.to_string()).collect()))
}

pub async fn get_random(db: Extension<Database>) -> Result<Json<Kanji>, AppError> {
    let mut cursor = db
        .collection::<Kanji>("kanjidic")
        .aggregate(
            [doc! {
                "$sample": {
                    "size": 1
                }
            }],
            None,
        )
        .await?
        .with_type::<Kanji>();

    if let Some(c) = cursor.try_next().await? {
        return Ok(Json(c));
    }

    Err(AppError::Error("No thing".into()))
}

pub async fn get_kanji(
    Path(kanji): Path<String>,
    db: Extension<Database>,
) -> Result<Json<Kanji>, AppError> {
    let out = db
        .collection::<Kanji>("kanjidic")
        .find_one(doc! { "literal": kanji}, None)
        .await?;

    Ok(Json(out.unwrap()))
}

#[derive(Deserialize)]
pub struct DictQuery {
    pub dict: String,
    pub entry: String,
}

pub async fn get_dict_entry(
    filter: Query<DictQuery>,
    db: Extension<Database>,
) -> Result<Json<Kanji>, AppError> {
    // use unwrap_or() for these

    let out = db
        .collection::<Kanji>("kanjidic")
        .find_one(
            doc! { "dict": {
                "$elemMatch": {
                    "type": &filter.dict,
                    "value": &filter.entry,
                },
            }},
            None,
        )
        .await?;

    Ok(Json(out.unwrap()))
}

#[derive(Deserialize)]
pub struct DictEntries {
    pub dict: String,
    pub from: Option<i64>,
    pub count: Option<i64>,
}

pub async fn get_dict_entries(
    params: Query<DictEntries>,
    db: Extension<Database>,
) -> Result<Json<Vec<Kanji>>, AppError> {
    let from = params.from.unwrap_or(0);
    let count = params.count.unwrap_or(10);

    let collation = Collation::builder()
        .locale("en_US")
        .numeric_ordering(true)
        .build();
    // let options = AggregateOptions::builder().collation(collation).build();

    let find_options = FindOptions::builder()
        .sort(doc! { "dict.value": 1})
        .skip(from as u64)
        .limit(count)
        .collation(collation)
        .build();

    let out = db
        .collection::<Kanji>("kanjidic")
        .find(
            doc! {
                "dict.type": &params.dict,
            },
            find_options,
        )
        // .aggregate(
        // sorting on this is pure torture
        // maybe this would be easier using $project?
        // [
        // doc! {
        //     "$match" : {
        //         "dict": {
        //             "$elemMatch": {
        //                 "type": &params.dict,
        //             },
        //         }
        //     }
        // },
        // // doc! {
        // //     "$unwind": "$dict",
        // // },
        // doc! {
        //     "$filter": {
        //         input: "dict",
        //     }
        // }
        // doc! {
        //     "$match": {
        //         "dict.type": &params.dict,
        //     }
        // },
        // doc! {
        //     "$sort": {
        //         "dict.value": 1,
        //     }
        // },
        // doc! {
        //     "$limit": from + count,
        // },
        // doc! {
        //     "$skip": from,
        // },
        // ],
        // options,
        // )
        .await?
        .with_type::<Kanji>();

    Ok(Json(out.try_collect().await?))
}

#[derive(Deserialize)]
pub struct SearchParams {
    pub search: String,
    pub from: Option<i64>,
    pub count: Option<i64>,
}

pub async fn get_search(
    params: Query<SearchParams>,
    db: Extension<Database>,
) -> Result<Json<Vec<Kanji>>, AppError> {
    // use unwrap_or() for these

    let from = params.from.unwrap_or(0) as u64;
    let count = params.count.unwrap_or(10);
    let find_options = FindOptions::builder()
        .sort(doc! { "literal": 1})
        .skip(from)
        .limit(count)
        .build();

    let out = db
        .collection::<Kanji>("kanjidic")
        .find(
            doc! { "meanings": {
            "$elemMatch": {
                "value": &params.search    } }},
            find_options,
        )
        .await?;

    Ok(Json(out.try_collect().await?))
}
