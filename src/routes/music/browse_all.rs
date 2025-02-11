use crate::core::app_state::AppState;
use axum::{
	extract::{Path, State},
	http::{header, StatusCode},
	response::Response,
};
use diesel::prelude::*;

pub async fn browse_all(State(app_state): State<AppState>, Path(category): Path<String>) -> Response<String> {
	let mut db_conn = match app_state.db_pool.get() {
		Ok(conn) => conn,
		Err(err) => {
			let msg = format!("Failed to get DB from pool: {err}");
			return Response::builder()
				.status(StatusCode::INTERNAL_SERVER_ERROR)
				.body(msg)
				.unwrap();
		}
	};

	use crate::schema::music::dsl::*;

	let result = match category.as_str() {
		"artists" => music.select(artist).distinct().load::<String>(&mut db_conn),
		"albums" => music.select(album).distinct().load::<String>(&mut db_conn),
		"genres" => music.select(genre).distinct().load::<String>(&mut db_conn),
		_ => {
			return Response::builder()
				.status(StatusCode::BAD_REQUEST)
				.body("Invalid category. Use 'artists', 'albums', or 'genres'.".to_string())
				.unwrap();
		}
	};

	match result {
		Ok(items) => match serde_json::to_string(&items) {
			Ok(json) => Response::builder()
				.status(StatusCode::OK)
				.header(header::CONTENT_TYPE, "application/json")
				.body(json)
				.unwrap(),
			Err(err) => Response::builder()
				.status(StatusCode::INTERNAL_SERVER_ERROR)
				.body(format!("Failed to serialize response: {err}"))
				.unwrap(),
		},
		Err(err) => Response::builder()
			.status(StatusCode::INTERNAL_SERVER_ERROR)
			.body(format!("Database error: {err}"))
			.unwrap(),
	}
}
