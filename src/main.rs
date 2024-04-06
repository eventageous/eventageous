use shuttle_secrets::SecretStore;

#[shuttle_runtime::main]
async fn main(#[shuttle_secrets::Secrets] secret_store: SecretStore) -> shuttle_axum::ShuttleAxum {
    eventageous::eventageous(secret_store).await
}
