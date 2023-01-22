use crate::{datetime::DateTime, Map};
use parking_lot::RwLock;
use std::sync::LazyLock;
use sysinfo::{DiskExt, NetworkExt, NetworksExt, System, SystemExt};

/// Refreshes the system and retrieves the information.
pub(super) fn refresh_and_retrieve() -> Map {
    // Refreshes the system first.
    refresh_system();

    // Reads the system.
    let sys = GLOBAL_MONITOR.read();
    let mut map = SYSTEM_INFO.clone();

    // Retrieves OS information.
    map.insert("os.uptime".to_owned(), sys.uptime().into());

    // Retrieves the system load average value.
    if sys
        .name()
        .is_some_and(|sys_name| !sys_name.eq_ignore_ascii_case("windows"))
    {
        let load_avg = sys.load_average();
        let load_avg_values = vec![load_avg.one, load_avg.five, load_avg.fifteen];
        map.insert("os.load_average".to_owned(), load_avg_values.into());
    }

    // Retrieves RAM and SWAP usage.
    map.insert("mem.free_memory".to_owned(), sys.free_memory().into());
    map.insert(
        "mem.available_memory".to_owned(),
        sys.available_memory().into(),
    );
    map.insert("mem.used_memory".to_owned(), sys.used_memory().into());
    map.insert("mem.free_swap".to_owned(), sys.free_swap().into());
    map.insert("mem.used_swap".to_owned(), sys.used_swap().into());

    // Retrieves the disks list.
    map.insert(
        "disk.available_space".to_owned(),
        sys.disks()
            .iter()
            .fold(0, |sum, disk| sum + disk.available_space())
            .into(),
    );

    // Retrieves the networks list.
    let mut network_received = 0;
    let mut network_total_received = 0;
    let mut network_transmitted = 0;
    let mut network_total_transmitted = 0;
    let mut network_packets_received = 0;
    let mut network_total_packets_received = 0;
    let mut network_packets_transmitted = 0;
    let mut network_total_packets_transmitted = 0;
    let mut network_errors_on_received = 0;
    let mut network_total_errors_on_received = 0;
    let mut network_errors_on_transmitted = 0;
    let mut network_total_errors_on_transmitted = 0;
    for (_name, network) in sys.networks() {
        network_received += network.received();
        network_total_received += network.total_received();
        network_transmitted += network.transmitted();
        network_total_transmitted += network.total_transmitted();
        network_packets_received += network.packets_received();
        network_total_packets_received += network.total_packets_received();
        network_packets_transmitted += network.packets_transmitted();
        network_total_packets_transmitted += network.total_packets_transmitted();
        network_errors_on_received += network.errors_on_received();
        network_total_errors_on_received += network.total_errors_on_received();
        network_errors_on_transmitted += network.errors_on_transmitted();
        network_total_errors_on_transmitted += network.total_errors_on_transmitted();
    }
    map.insert("net.received".to_owned(), network_received.into());
    map.insert(
        "net.total_received".to_owned(),
        network_total_received.into(),
    );
    map.insert("net.transmitted".to_owned(), network_transmitted.into());
    map.insert(
        "net.total_transmitted".to_owned(),
        network_total_transmitted.into(),
    );
    map.insert(
        "net.packets_received".to_owned(),
        network_packets_received.into(),
    );
    map.insert(
        "net.total_packets_received".to_owned(),
        network_total_packets_received.into(),
    );
    map.insert(
        "net.packets_transmitted".to_owned(),
        network_packets_transmitted.into(),
    );
    map.insert(
        "net.total_packets_transmitted".to_owned(),
        network_total_packets_transmitted.into(),
    );
    map.insert(
        "net.errors_on_received".to_owned(),
        network_errors_on_received.into(),
    );
    map.insert(
        "net.total_errors_on_received".to_owned(),
        network_total_errors_on_received.into(),
    );
    map.insert(
        "net.errors_on_transmitted".to_owned(),
        network_errors_on_transmitted.into(),
    );
    map.insert(
        "net.total_errors_on_transmitted".to_owned(),
        network_total_errors_on_transmitted.into(),
    );

    map
}

/// Refreshes the system.
fn refresh_system() {
    let mut sys = GLOBAL_MONITOR.write();
    sys.refresh_cpu();
    sys.refresh_memory();
    sys.refresh_disks_list();
    sys.networks_mut().refresh_networks_list();
}

/// Static system information.
static SYSTEM_INFO: LazyLock<Map> = LazyLock::new(|| {
    let mut map = Map::new();
    let mut sys = System::new();
    sys.refresh_cpu();
    sys.refresh_memory();
    sys.refresh_disks_list();

    // Retrieves OS information.
    map.insert("os.name".to_owned(), sys.name().into());
    map.insert("os.version".to_owned(), sys.os_version().into());
    if let Ok(boot_time) = i64::try_from(sys.boot_time()) {
        let booted_at = DateTime::from_timestamp(boot_time);
        map.insert("os.booted_at".to_owned(), booted_at.to_string().into());
    }

    // Retrieves CPUs information.
    map.insert("cpu.num_cpus".to_owned(), sys.cpus().len().into());
    map.insert(
        "cpu.physical_core_count".to_owned(),
        sys.physical_core_count().into(),
    );

    // Retrieves RAM and SWAP information.
    map.insert("mem.total_memory".to_owned(), sys.total_memory().into());
    map.insert("mem.total_swap".to_owned(), sys.total_swap().into());

    // Retrieves the disks list.
    map.insert(
        "disk.total_space".to_owned(),
        sys.disks()
            .iter()
            .fold(0, |sum, disk| sum + disk.total_space())
            .into(),
    );

    map
});

/// Global system monitor.
static GLOBAL_MONITOR: LazyLock<RwLock<System>> = LazyLock::new(|| RwLock::new(System::new()));
