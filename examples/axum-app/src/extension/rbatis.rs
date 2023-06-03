use rbatis::RBatis;
use rbdc_mysql::driver::MysqlDriver;
use std::sync::LazyLock;
use zino::{prelude::*, Cluster};

pub static RBATIS: LazyLock<RBatis> = LazyLock::new(|| {
    let config = Cluster::config()
        .get_first_table("mysql")
        .expect("the `mysql` field should be a nonempty array of tables");
    let database = config
        .get_str("database")
        .expect("the `database` field should be a str");
    let authority = State::format_authority(config, Some(3306));
    let dsn = format!("mysql://{authority}/{database}");

    let rb = RBatis::new();
    rb.init(MysqlDriver {}, &dsn)
        .expect("fail to init the RBatis instance");
    rb
});
