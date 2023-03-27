use std::str::FromStr;

use poem::{listener::TcpListener, Route, Server};
use try_poem::TenantId;

use poem_openapi::{payload::Json, ApiResponse, Object, OpenApi, OpenApiService, Tags};
use rand::Rng;

#[derive(Tags)]
enum ApiTags {
    /// Operations about user
    Tenant,
}

#[derive(Object)]
pub struct ErrorBody {
    pub msg: String,
}

#[derive(Object, Default, Debug)]
pub struct TenantCreateRequest {
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

#[derive(Object)]
struct CreateTenantOkResponse {
    id: TenantId,
}

#[derive(ApiResponse)]
enum CreateTenantResponse {
    /// Returns when the user is successfully created.
    #[oai(status = 201)]
    Ok(Json<CreateTenantOkResponse>),

    #[oai(status = 400)]
    BadRequestError(Json<ErrorBody>),

    #[oai(status = 404)]
    NotFoundError(Json<ErrorBody>),

    #[oai(status = 500)]
    InternalErr(Json<ErrorBody>),
}

#[derive(Default)]
struct Api {}

#[OpenApi(prefix_path = "/v1")]
impl Api {
    /// Create a tenant. Returns new tenant id on success.
    /// If no new tenant id is specified in parameters, it would be generated. It's an error to recreate the same tenant.
    #[oai(path = "/tenant", method = "post", tag = "ApiTags::Tenant")]
    async fn create_tenant(&self, tenant: Json<TenantCreateRequest>) -> CreateTenantResponse {
        use CreateTenantResponse::*;
        let mut rng = rand::thread_rng();
        println!("{tenant:?}");
        let n = rng.gen_range(0..10);
        if n < 7 {
            let id = TenantId::from_str("9840a3586d1a413699627b1dcf3e5103").unwrap();
            return Ok(Json(CreateTenantOkResponse { id }));
        }
        // NOTE: its impossible to write generic impl From<FooError> for ApiError
        //       It is a different type for every endpoint.
        match n {
            7 => BadRequestError(Json(ErrorBody {
                msg: "BadRequest".to_owned(),
            })),
            8 => NotFoundError(Json(ErrorBody {
                msg: "NotFoundError".to_owned(),
            })),
            9 => InternalErr(Json(ErrorBody {
                msg: "InternalErr".to_owned(),
            })),
            _ => panic!("uh oh"),
        }
    }
}

// curl -XPOST \
//     -H 'accept: application/json; charset=utf-8' \
//     -H 'Content-Type: application/json; charset=utf-8' \
//     http://localhost:3000/api/v1/tenant \
//     -d"{\"new_tenant_id\": null}"

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "poem=debug");
    }
    tracing_subscriber::fmt::init();

    let api_service =
        OpenApiService::new(Api::default(), "Users", "1.0").server("http://localhost:3000/api");
    let ui = api_service.swagger_ui();

    Server::new(TcpListener::bind("127.0.0.1:3000"))
        .run(Route::new().nest("/api", api_service).nest("/", ui))
        .await
}
