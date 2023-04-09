use crate::{datetime::DateTime, extension::JsonObjectExt, Map};
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
    map.upsert("os.uptime", sys.uptime());

    // Retrieves the system load average value.
    if sys
        .name()
        .is_some_and(|sys_name| !sys_name.eq_ignore_ascii_case("windows"))
    {
        let load_avg = sys.load_average();
        let load_avg_values = vec![load_avg.one, load_avg.five, load_avg.fifteen];
        map.upsert("os.load_average", load_avg_values);
    }

    // Retrieves RAM and SWAP usage.
    map.upsert("mem.free_memory", sys.free_memory());
    map.upsert("mem.available_memory", sys.available_memory());
    map.upsert("mem.used_memory", sys.used_memory());
    map.upsert("mem.free_swap", sys.free_swap());
    map.upsert("mem.used_swap", sys.used_swap());

    // Retrieves the disks list.
    map.upsert(
        "disk.available_space",
        sys.disks()
            .iter()
            .fold(0, |sum, disk| sum + disk.available_space()),
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
    map.upsert("net.received", network_received);
    map.upsert("net.total_received", network_total_received);
    map.upsert("net.transmitted", network_transmitted);
    map.upsert("net.total_transmitted", network_total_transmitted);
    map.upsert("net.packets_received", network_packets_received);
    map.upsert("net.total_packets_received", network_total_packets_received);
    map.upsert("net.packets_transmitted", network_packets_transmitted);
    map.upsert(
        "net.total_packets_transmitted",
        network_total_packets_transmitted,
    );
    map.upsert("net.errors_on_received", network_errors_on_received);
    map.upsert(
        "net.total_errors_on_received",
        network_total_errors_on_received,
    );
    map.upsert("net.errors_on_transmitted", network_errors_on_transmitted);
    map.upsert(
        "net.total_errors_on_transmitted",
        network_total_errors_on_transmitted,
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
    map.upsert("os.name", sys.name());
    map.upsert("os.version", sys.os_version());
    if let Ok(boot_time) = i64::try_from(sys.boot_time()) {
        map.upsert("os.booted_at", DateTime::from_timestamp(boot_time));
    }

    // Retrieves CPUs information.
    map.upsert("cpu.num_cpus", sys.cpus().len());
    map.upsert("cpu.physical_core_count", sys.physical_core_count());

    // Retrieves RAM and SWAP information.
    map.upsert("mem.total_memory", sys.total_memory());
    map.upsert("mem.total_swap", sys.total_swap());

    // Retrieves the disks list.
    map.upsert(
        "disk.total_space",
        sys.disks()
            .iter()
            .fold(0, |sum, disk| sum + disk.total_space()),
    );

    map
});

/// Global system monitor.
static GLOBAL_MONITOR: LazyLock<RwLock<System>> = LazyLock::new(|| RwLock::new(System::new()));
