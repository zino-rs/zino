use crate::model::User;
use zino::prelude::*;

pub fn create_initial_account(_ctx: &mut JobContext) -> BoxFuture<'_> {
    let mut query = User::default_query();
    query.add_filter("roles", "admin");
    Box::pin(async move {
        if User::count(&query).await == Ok(0) {
            let mut admin = User::new();
            let mut data = Map::new();
            data.upsert("name", "Administrator");
            data.upsert("roles", "admin");
            data.upsert("account", "admin");
            data.upsert("password", "admin");
            if admin.read_map(&data).is_success()
                && let Err(err) = admin.insert().await
            {
                tracing::error!("fail to create initial account: {err}");
            }
        }
    })
}
