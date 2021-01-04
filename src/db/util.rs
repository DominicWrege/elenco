use deadpool_postgres::{Manager, Pool};
use tokio_postgres::{tls::NoTlsStream, Socket};

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("./sql/migrations");
}

fn default_port() -> u16 {
    5432
}

#[derive(serde::Deserialize, Debug)]
struct DBConfig {
    username: String,
    password: String,
    //TODO id or url
    host: String,
    databasename: String,
    #[serde(default = "default_port")]
    port: u16,
}

impl DBConfig {
    pub fn new() -> Result<Self, envy::Error> {
        envy::prefixed("DB_").from_env::<DBConfig>()
    }
}

struct DBContext {
    client: tokio_postgres::Client,
    connection: tokio_postgres::Connection<Socket, NoTlsStream>,
    config: tokio_postgres::Config,
}

async fn connect_with_conf() -> Result<DBContext, anyhow::Error> {
    let mut config = tokio_postgres::Config::default();
    let db_config = DBConfig::new()?;
    config
        .user(&db_config.username)
        .password(&db_config.password)
        .dbname(&db_config.databasename)
        .host(&db_config.host);
    let (client, connection) = config.connect(tokio_postgres::NoTls).await?;
    Ok(DBContext {
        client,
        connection,
        config,
    })
}

pub async fn connect_and_migrate() -> Result<Pool, anyhow::Error> {
    let DBContext {
        mut client,
        connection,
        config,
    } = connect_with_conf().await?;
    tokio::task::spawn(connection);
    embedded::migrations::runner()
        .run_async(&mut client)
        .await?;
    let mngr = Manager::new(config.clone(), tokio_postgres::NoTls);
    Ok(Pool::new(mngr, 12))
}
