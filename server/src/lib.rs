use grpc::smm::{
    self, posts::posts_service_server::PostsServiceServer,
    users::users_service_server::UsersServiceServer,
};
use posts::AppPostService;
use users::AppUsersService;

mod posts;
mod users;

pub async fn run() -> anyhow::Result<()> {
    let addr = std::env::var("GRPC_SERVER_ADDR")?;
    let subscriber = tracing_subscriber::fmt()
        .pretty()
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(false)
        .with_target(false)
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;
    tracing::info!(message = "Starting server", %addr);
    let reflection_service_v1 = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(smm::FILE_DESCRIPTOR_SET)
        .build_v1()?;
    let reflection_service_alpha = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(smm::FILE_DESCRIPTOR_SET)
        .build_v1alpha()?;
    let users_service = UsersServiceServer::with_interceptor(AppUsersService::new(), check_auth);
    let posts_service = PostsServiceServer::with_interceptor(AppPostService::new(), check_auth);
    tonic::transport::Server::builder()
        .trace_fn(|_| tracing::info_span!("smm"))
        .add_service(reflection_service_v1)
        .add_service(reflection_service_alpha)
        .add_service(users_service)
        .add_service(posts_service)
        .serve(addr.parse()?)
        .await?;
    Ok(())
}

fn check_auth(req: tonic::Request<()>) -> tonic::Result<tonic::Request<()>> {
    let token: tonic::metadata::MetadataValue<_> = "Bearer some-secret-token".parse().unwrap();

    match req.metadata().get("authorization") {
        Some(t) if token == t => Ok(req),
        _ => Err(tonic::Status::unauthenticated("No valid auth token")),
    }
}
