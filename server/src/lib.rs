use grpc::smm::{
    self, posts::posts_service_server::PostsServiceServer,
    users::users_service_server::UsersServiceServer,
};
use posts::AppPostService;
use users::AppUsersService;

mod posts;
mod users;

use clap::Parser;

#[derive(Parser)]
#[command(name = "SMMaster server", version, about = "gRPC server for SMM telegram bot", long_about = None)]
struct Cli {
    /// Define port to serve
    #[arg(short, long)]
    port: Option<u16>,
    /// MongoDB URI
    #[arg(short, long)]
    database: Option<String>,
}

pub async fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let port = cli.port.unwrap_or(50052);
    let mongo_db_uri = cli
        .database
        .unwrap_or(String::from("mongodb://localhost:27017"));
    let addr = format!("[::1]:{port}");
    let subscriber = tracing_subscriber::fmt()
        .pretty()
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(false)
        .with_target(false)
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;
    tracing::info!(message = "Starting server", %addr);
    let db = storage::Storage::new(&mongo_db_uri).await?;
    let reflection_service_v1 = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(smm::FILE_DESCRIPTOR_SET)
        .build_v1()?;
    let reflection_service_alpha = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(smm::FILE_DESCRIPTOR_SET)
        .build_v1alpha()?;
    let users_service =
        UsersServiceServer::with_interceptor(AppUsersService::new(db.clone()), check_auth);
    let posts_service = PostsServiceServer::with_interceptor(AppPostService::new(db), check_auth);
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
