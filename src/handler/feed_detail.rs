use crate::{
    db::rows_into_vec,
    inc_sql,
    model::{json::Feed, Account, Permission},
    session_storage, template,
    util::redirect,
    State,
};

use actix_web::{web, HttpResponse};
use askama::Template;
use template::FeedDetailSite;

use super::general_error::GeneralError;

pub async fn site(
    state: web::Data<State>,
    session: actix_session::Session,
    id: actix_web::web::Path<i32>,
) -> Result<HttpResponse, GeneralError> {
    let feed_id = id.into_inner();
    let client = state.db_pool.get().await?;
    let account = Account::from_session(&session).unwrap();

    // moderator can access all feeds and submitter only there own
    if account.permission() != Permission::Admin {
        let submitter_check_stmnt = client.prepare(inc_sql!("get/feed/submitter_check")).await?;
        if let None = client
            .query_opt(&submitter_check_stmnt, &[&feed_id, &account.id()])
            .await?
        {
            return Ok(redirect("404"));
        }
    }

    let feed_stmnt = client.prepare(inc_sql!("get/feed/preview_by_id")).await?;
    if let Ok(feed_row) = client.query_one(&feed_stmnt, &[&feed_id]).await {
        let epsiodes_stmnt = client
            .prepare(inc_sql!("get/episodes_small_for_feed_id"))
            .await?;
        let episodes = rows_into_vec(client.query(&epsiodes_stmnt, &[&feed_id]).await?);
        if let Ok(feed) = Feed::from(&client, feed_row).await {
            let html = FeedDetailSite {
                permission: session_storage::permission(&session),
                feed,
                episodes,
            };
            return Ok(HttpResponse::Ok()
                .content_type("text/html")
                .body(html.render().unwrap()));
        }
    }

    Ok(redirect("404"))
}
