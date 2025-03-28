use zino::{Cluster, Request, Response, Result, prelude::*};

pub async fn index(req: Request) -> Result {
    let stats = json!({
        "method": "GET",
        "path": "/stats",
        "app_state_data": Cluster::state_data(),
    });
    let data = json!({
        "title": "Stats",
        "output": stats.to_string_pretty(),
    });
    let res = Response::default().context(&req);
    Ok(res.render("output.html", data).into())
}

pub async fn app_state(req: Request) -> Result {
    let mut page = InertiaPage::new("AppState").context(&req);
    page.append_props(&mut Cluster::state_data().to_owned());

    let mut res = Response::default().context(&req);
    res.send_inertia_page(page);
    Ok(res.into())
}
