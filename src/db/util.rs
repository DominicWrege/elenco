use deadpool_postgres::{Manager, Pool};
use tokio_postgres::{tls::NoTlsStream, Socket};

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("./migrations");
}

struct DBContext {
    client: tokio_postgres::Client,
    connection: tokio_postgres::Connection<Socket, NoTlsStream>,
    config: tokio_postgres::Config,
}

async fn connect_with_conf() -> Result<DBContext, anyhow::Error> {
    let mut config = tokio_postgres::Config::default();
    config
        .user("harra")
        .password("hund")
        .dbname("podcast")
        .host("127.0.0.1");
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
