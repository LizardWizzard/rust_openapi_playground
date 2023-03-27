use std::net::{Ipv4Addr, SocketAddr};

use axum::{routing, Router, Server};
use hyper::Error;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

mod tenant {
    use std::str::FromStr;

    use axum::{response::IntoResponse, Json};
    use hyper::StatusCode;
    use rand::Rng;
    use serde::{Deserialize, Serialize};
    use serde_with::{serde_as, DisplayFromStr};
    use try_utoipa::TenantId;
    use utoipa::ToSchema;

    #[serde_as]
    #[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
    pub(super) struct TenantCreateRequest {
        #[serde_as(as = "Option<DisplayFromStr>")]
        pub new_tenant_id: Option<TenantId>,
        pub checkpoint_distance: Option<u64>,
        pub checkpoint_timeout: Option<String>,
        pub compaction_target_size: Option<u64>,
        pub compaction_period: Option<String>,
        pub compaction_threshold: Option<usize>,
        pub gc_horizon: Option<u64>,
        pub gc_period: Option<String>,
        pub image_creation_threshold: Option<usize>,
        pub pitr_interval: Option<String>,
        pub walreceiver_connect_timeout: Option<String>,
        pub lagging_wal_timeout: Option<String>,
        // pub max_lsn_wal_lag: Option<NonZeroU64>, TODO need to implement
        pub trace_read_requests: Option<bool>,
    }

    /// Todo operation errors
    #[derive(Serialize, Deserialize, ToSchema)]
    pub(super) enum TenantError {
        #[schema(example = "Bad request")]
        BadRequest(String),
        #[schema(example = "id = 1")]
        NotFound(String),
        #[schema(example = "uh oh")]
        InternalErr(String),
    }

    #[serde_as]
    #[derive(Serialize, Deserialize, ToSchema)]
    pub struct CreateTenantResponse {
        #[serde_as(as = "DisplayFromStr")]
        id: TenantId,
    }

    #[derive(Serialize, Deserialize, ToSchema)]
    pub struct ErrorBody {
        pub msg: String,
    }

    /// Create new Todo
    ///
    /// Tries to create a new Todo item to in-memory storage or fails with 409 conflict if already exists.
    #[utoipa::path(
        post,
        path = "/api/v1/tenant",
        request_body = TenantCreateRequest,
        responses(
            (status = 201, description = "Tenant created successfully", body = CreateTenantResponse),
            (status = 400, description = "Bad tenant", body = TenantError)
        )
    )]
    pub(super) async fn create(Json(tenant): Json<TenantCreateRequest>) -> impl IntoResponse {
        let mut rng = rand::thread_rng();
        println!("{tenant:?}");
        let n = rng.gen_range(0..10);
        if n < 7 {
            let id = TenantId::from_str("9840a3586d1a413699627b1dcf3e5103").unwrap();
            return (StatusCode::CREATED, Json(CreateTenantResponse { id })).into_response();
        }
        // NOTE: its impossible to write generic impl From<FooError> for ApiError
        //       It is a different type for every endpoint.
        let (status, body) = match n {
            7 => (
                StatusCode::BAD_REQUEST,
                Json(ErrorBody {
                    msg: "BAD_REQUEST".to_owned(),
                }),
            ),
            8 => (
                StatusCode::NOT_FOUND,
                Json(ErrorBody {
                    msg: "NOT_FOUND".to_owned(),
                }),
            ),
            9 => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorBody {
                    msg: "INTERNAL_SERVER_ERROR".to_owned(),
                }),
            ),
            _ => panic!("uh oh"),
        };

        (status, body).into_response()
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    #[derive(OpenApi)]
    #[openapi(
        paths(
            tenant::create,
        ),
        components(
            schemas(tenant::TenantCreateRequest, tenant::TenantError, tenant::CreateTenantResponse, try_utoipa::TenantId, try_utoipa::Id)
        ),
        tags(
            (name = "todo", description = "Todo items management API")
        )
    )]
    struct ApiDoc;

    let app = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-doc/openapi.json", ApiDoc::openapi()))
        .route("/api/v1/tenant", routing::post(tenant::create));

    let address = SocketAddr::from((Ipv4Addr::UNSPECIFIED, 3000));
    Server::bind(&address).serve(app.into_make_service()).await
}
