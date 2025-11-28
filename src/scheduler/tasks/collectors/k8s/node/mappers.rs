/* Maps kubelet Summary DTO → internal models */

use crate::core::persistence::info::k8s::node::info_node_entity::InfoNodeEntity;
use crate::core::persistence::metrics::k8s::node::metric_node_entity::MetricNodeEntity;
use crate::scheduler::tasks::collectors::k8s::summary_dto::{NetworkStats, Summary};
use chrono::Utc;

pub fn map_summary_to_node_info(summary: &Summary) -> InfoNodeEntity {
    InfoNodeEntity {
        node_name: Some(summary.node.node_name.clone()),
        last_updated_info_at: Some(Utc::now()),
        ready: Some(true),
        ..Default::default() // leaves all other fields as None
    }
}

pub fn map_summary_to_metrics(summary: &Summary) -> MetricNodeEntity {
    let n = &summary.node;

    // --- Compute summed physical network stats ---
    let (rx, tx, rx_err, tx_err) = n.network.as_ref()
        .and_then(|net| sum_network_interfaces(net))
        .unwrap_or((None, None, None, None));

    MetricNodeEntity {
        time: Utc::now(),

        // CPU
        cpu_usage_nano_cores: n.cpu.usage_nano_cores,
        cpu_usage_core_nano_seconds: n.cpu.usage_core_nano_seconds,

        // Memory
        memory_usage_bytes: n.memory.usage_bytes,
        memory_working_set_bytes: n.memory.working_set_bytes,
        memory_rss_bytes: n.memory.rss_bytes,
        memory_page_faults: n.memory.page_faults,

        // Network (physical)
        network_physical_rx_bytes: rx,
        network_physical_tx_bytes: tx,
        network_physical_rx_errors: rx_err,
        network_physical_tx_errors: tx_err,

        // Filesystem
        fs_used_bytes: n.fs.as_ref().and_then(|x| x.used_bytes),
        fs_capacity_bytes: n.fs.as_ref().and_then(|x| x.capacity_bytes),
        fs_inodes_used: n.fs.as_ref().and_then(|x| x.inodes_used),
        fs_inodes: n.fs.as_ref().and_then(|x| x.inodes),
    }
}

fn sum_network_interfaces(net: &NetworkStats) -> Option<(Option<u64>, Option<u64>, Option<u64>, Option<u64>)> {
    net.interfaces.as_ref().map(|interfaces| {
        let (rx, tx, rx_err, tx_err) = interfaces.iter().fold((0, 0, 0, 0), |acc, iface| {
            (
                acc.0 + iface.rx_bytes.unwrap_or(0),
                acc.1 + iface.tx_bytes.unwrap_or(0),
                acc.2 + iface.rx_errors.unwrap_or(0),
                acc.3 + iface.tx_errors.unwrap_or(0),
            )
        });
        (Some(rx), Some(tx), Some(rx_err), Some(tx_err))
    })
}
