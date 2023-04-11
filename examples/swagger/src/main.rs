use std::env;

use dotenv::dotenv;
use rustic::{WithLogging, Redirect};
use rustic_sqlx::WithSQLx;
use rustic_swagger::WithSwagger;
use sqlx::{SqlitePool, Sqlite};
use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
    Modify, OpenApi,
};

#[async_std::main]
async fn main() -> std::io::Result<()> {
    femme::start();
    dotenv().ok();

    #[derive(OpenApi)]
    #[openapi(
        paths(
            todo::list_todos,
            todo::create_todo,
            todo::delete_todo,
            todo::mark_done
        ),
        components(
            schemas(todo::Todo, todo::TodoError)
        ),
        modifiers(&SecurityAddon),
        tags(
            (name = "todo", description = "Todo items management endpoints.")
        )
    )]
    struct ApiDoc;

    struct SecurityAddon;

    impl Modify for SecurityAddon {
        fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
            let components = openapi.components.as_mut().unwrap(); // we can unwrap safely since there already is components registered.
            components.add_security_scheme(
                "api_key",
                SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("todo_apikey"))),
            )
        }
    }

    let pool = SqlitePool::connect(&env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();

    sqlx::migrate!("./migrations").run(&pool).await.unwrap();

    let mut app = rustic::new();

    app.with_logging();
    app.with_sqlx::<Sqlite>(&env::var("DATABASE_URL").unwrap()).await;
    app.with_swagger(ApiDoc::openapi());

    app.at("youtube").get(Redirect::new("https://www.youtube.com/watch?v=dQw4w9WgXcQ"));

    app.at("api/todo").get(todo::list_todos);
    app.at("api/todo").post(todo::create_todo);
    app.at("api/todo/:id").delete(todo::delete_todo);
    app.at("api/todo/:id").put(todo::mark_done);

    app.listen("0.0.0.0:8080").await
}

mod todo {
    use std::env;

    use http_types::StatusCode;
    use rustic::{Request, Response};
    use serde::{Deserialize, Serialize};
    use serde_json::json;
    use sqlx::{Row, SqlitePool};
    use utoipa::ToSchema;

    /// Item to complete
    #[derive(Serialize, Deserialize, ToSchema, Clone)]
    pub(super) struct Todo {
        /// Unique database id for `Todo`
        #[schema(example = 1)]
        id: i32,
        /// Description of task to complete
        #[schema(example = "Buy coffee")]
        value: String,
        /// Indicates whether task is done or not
        done: bool,
    }

    /// Error that might occur when managing `Todo` items
    #[derive(Serialize, Deserialize, ToSchema)]
    pub(super) enum TodoError {
        /// Happens when Todo item already exists
        Config(String),
        /// Todo not found from storage
        NotFound(String),
    }

    /// List todos from in-memory storage.
    ///
    /// List all todos from in memory storage.
    #[utoipa::path(
        get,
        path = "/api/todo",
        responses(
            (status = 200, description = "List all todos successfully", body = [Todo])
        )
    )]
    pub(super) async fn list_todos(_req: Request) -> rustic::Result<Response> {
        let pool = SqlitePool::connect(&env::var("DATABASE_URL")?).await?;

        sqlx::migrate!("./migrations").run(&pool).await.unwrap();

        let recs = sqlx::query("SELECT id, description, done FROM todo ORDER BY id")
            .fetch_all(&pool)
            .await?;

        let mut todos = Vec::new();

        for rec in recs {
            todos.push(Todo {
                id: rec.get("id"),
                value: rec.get("description"),
                done: rec.get("done"),
            });
        }

        let mut response = Response::new(StatusCode::Ok);
        response.set_body(json!(todos));

        Ok(response)
    }

    /// Create new todo
    ///
    /// Create new todo to in-memory storage if not exists.
    #[utoipa::path(
        post,
        path = "/api/todo",
        request_body = Todo,
        responses(
            (status = 201, description = "Todo created successfully", body = Todo),
            (status = 409, description = "Todo already exists", body = TodoError, example = json!(TodoError::Config(String::from("id = 1"))))
        )
    )]
    pub(super) async fn create_todo(mut req: Request) -> rustic::Result<Response> {
        let new_todo = req.body_json::<Todo>().await?;

        let pool = SqlitePool::connect(&env::var("DATABASE_URL")?).await?;

        sqlx::migrate!("./migrations").run(&pool).await.unwrap();

        let query = "INSERT INTO todo (id, description, done) VALUES ($1, $2, $3)";

        sqlx::query(query)
            .bind(&new_todo.id)
            .bind(&new_todo.value)
            .bind(&new_todo.done)
            .execute(&pool)
            .await?;

        Ok(Response::new(StatusCode::Created))
    }

    /// Delete todo by id.
    ///
    /// Delete todo from in-memory storage.
    #[utoipa::path(
        delete,
        path = "/api/todo/{id}",
        responses(
            (status = 200, description = "Todo deleted successfully"),
            (status = 401, description = "Unauthorized to delete Todo"),
            (status = 404, description = "Todo not found", body = TodoError, example = json!(TodoError::NotFound(String::from("id = 1"))))
        ),
        params(
            ("id" = i32, Path, description = "Id of todo item to delete")
        ),
        security(
            ("api_key" = [])
        )
    )]
    pub(super) async fn delete_todo(req: Request) -> rustic::Result<Response> {
        let id = req.param("id")?.parse::<i32>()?;
        let api_key = req
            .header("todo_apikey")
            .map(|header| header.as_str().to_string())
            .unwrap_or_default();

        if api_key != "utoipa-rocks" {
            return Ok(Response::new(401));
        }

        let pool = SqlitePool::connect(&env::var("DATABASE_URL")?).await?;

        sqlx::migrate!("./migrations").run(&pool).await.unwrap();

        sqlx::query("DELETE FROM todo WHERE id = $id")
            .bind(id)
            .execute(&pool)
            .await?;

        let response = Response::new(StatusCode::NoContent);
        Ok(response)
    }

    /// Mark todo done by id
    #[utoipa::path(
        put,
        path = "/api/todo/{id}",
        responses(
            (status = 200, description = "Todo marked done successfully"),
            (status = 404, description = "Todo not found", body = TodoError, example = json!(TodoError::NotFound(String::from("id = 1"))))
        ),
        params(
            ("id" = i32, Path, description = "Id of todo item to mark done")
        )
    )]
    pub(super) async fn mark_done(req: Request) -> rustic::Result<Response> {
        let id = req.param("id")?.parse::<i32>()?;

        let pool = SqlitePool::connect(&env::var("DATABASE_URL")?).await?;

        sqlx::migrate!("./migrations").run(&pool).await.unwrap();

        sqlx::query("UPDATE todo SET done = 1 WHERE id = $id")
            .bind(id)
            .execute(&pool)
            .await?;

        let response = Response::new(StatusCode::NoContent);
        Ok(response)
    }
}
