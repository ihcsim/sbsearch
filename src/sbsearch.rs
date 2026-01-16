use chrono::{self, DateTime, Utc};
use grep_matcher::Matcher;
use grep_regex::RegexMatcher;
use grep_searcher::{Searcher, SearcherBuilder, sinks::UTF8};
use std::error::Error;
use std::fmt;
use std::fs::{self};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Entry {
    pub level: String,
    pub path: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
}

pub struct SearchResult {
    pub entries_offset: Vec<Entry>,
}

impl fmt::Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let out = self.content.clone();
        write!(f, "{}", out)
    }
}

pub fn search(
    dir: &Path,
    keyword: &str,
    offset: usize,
    limit: usize,
    cache: &mut Vec<Entry>,
) -> Result<SearchResult, Box<dyn Error>> {
    if cache.is_empty() {
        let mut sbsearch = SBSearch::new(keyword)?;
        sbsearch.search_tree(dir, cache)?;
        cache.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
    }

    let limit = limit.min(cache.len().saturating_sub(offset));
    let entries_offset = cache.iter().skip(offset).take(limit).cloned().collect();

    Ok(SearchResult { entries_offset })
}

struct SBSearch {
    searcher: Searcher,
    matcher_keyword: RegexMatcher,
    matcher_log_level1: RegexMatcher,
    matcher_log_level2: RegexMatcher,
    matcher_log_level3: RegexMatcher,
    matcher_log_level4: RegexMatcher,
    matcher_timestamp: RegexMatcher,
}

impl SBSearch {
    fn new(keyword: &str) -> Result<Self, Box<dyn Error>> {
        let searcher: Searcher;
        unsafe {
            let mmap_choice = grep_searcher::MmapChoice::auto();
            searcher = SearcherBuilder::new()
                .memory_map(mmap_choice)
                .heap_limit(Some(268435456))
                .build();
        }
        let pattern = String::from(".*") + keyword + ".*";
        let matcher_keyword = RegexMatcher::new(pattern.as_str())?;
        let matcher_timestamp =
            RegexMatcher::new(r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?Z")?;
        let matcher_log_level1 = RegexMatcher::new(r"level=([^\s]+)")?;
        let matcher_log_level2 = RegexMatcher::new(r#""level":"([^"]+)""#)?;
        let matcher_log_level3 = RegexMatcher::new(r"err=")?;
        let matcher_log_level4 = RegexMatcher::new(r"(?i)\[error\]")?;
        Ok(SBSearch {
            searcher,
            matcher_keyword,
            matcher_log_level1,
            matcher_log_level2,
            matcher_log_level3,
            matcher_log_level4,
            matcher_timestamp,
        })
    }

    fn search_tree(&mut self, dir: &Path, entries: &mut Vec<Entry>) -> Result<(), Box<dyn Error>> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                self.search_tree(&path, entries)?;
                continue;
            }

            if path.is_file() {
                let searcher = &mut self.searcher.clone();
                self.search_file(&path, entries, searcher)?;
                continue;
            }

            println!("skipping {}", path.display())
        }
        Ok(())
    }

    fn search_file(
        &self,
        path: &Path,
        entries: &mut Vec<Entry>,
        searcher: &mut Searcher,
    ) -> Result<(), Box<dyn Error>> {
        searcher.search_path(
            &self.matcher_keyword,
            path,
            UTF8(|_lnum, line| {
                let timestamp = self.matcher_timestamp.find(line.as_bytes())?.unwrap();
                let timestamp_fixed_offset =
                    DateTime::parse_from_rfc3339(&line[timestamp]).unwrap();
                let mut level = "UNKNOWN";
                if let Ok(r) = self.find_log_level(line) {
                    level = r;
                }

                let entry = Entry {
                    content: String::from(line),
                    level: String::from(level),
                    path: String::from(path.to_str().unwrap()),
                    timestamp: timestamp_fixed_offset.with_timezone(&Utc),
                };
                entries.push(entry);
                Ok(true)
            }),
        )?;
        Ok(())
    }

    fn find_log_level<'a>(&self, line: &'a str) -> Result<&'a str, Box<dyn Error>> {
        if let Ok(opt) = self.matcher_log_level1.find(line.as_bytes())
            && let Some(m) = opt
        {
            Ok(line[m.start()..m.end()].split('=').nth(1).unwrap())
        } else if let Ok(opt) = self.matcher_log_level2.find(line.as_bytes())
            && let Some(m) = opt
        {
            Ok(line[m.start()..m.end()]
                .split(':')
                .nth(1)
                .unwrap()
                .trim_matches('"'))
        } else if let Ok(opt) = self.matcher_log_level3.find(line.as_bytes())
            && opt.is_some()
        {
            Ok("error")
        } else if let Ok(opt) = self.matcher_log_level4.find(line.as_bytes())
            && opt.is_some()
        {
            Ok("error")
        } else {
            Ok("UNKNOWN")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_log_level_pattern1() {
        let sb_search = SBSearch::new("test").unwrap();

        let line = r#"2025-12-08T07:35:14.665171218Z ts=2025-12-08T07:35:14.665Z caller=kubernetes.go:331 level=info component="discovery manager scrape" discovery=kubernetes config=serviceMonitor/cattle-fleet-system/monitoring-fleet-controller/0 msg="Using pod service account via in-cluster config"#;
        let expected = "info";
        let actual = sb_search.find_log_level(line).unwrap();
        assert_eq!(actual, expected);

        let line = r#"2025-12-08T07:35:16.192939534Z time="2025-12-08T07:35:16Z" level=info msg="Diff: [docker.io/rancher/harvester-node-disk-manager-webhook:v0.7.11 docker.io/rancher/harvester:v1.4.3 docker.io/rancher/kubectl:v1.21.5 ghcr.io/k8snetworkplumbingwg/whereabouts:v0.7.0 docker.io/longhornio/csi-node-driver-registrar:v2.13.0 docker.io/longhornio/longhorn-cli:v1.7.3 docker.io/rancher/hardened-flannel:v0.26.5-build20250306 docker.io/rancher/harvester-network-controller:v0.5.6 docker.io/rancher/mirrored-jimmidyson-configmap-reload:v0.4.0 docker.io/rancher/system-agent-installer-rancher:v2.10.1 docker.io/rancher/system-agent:v0.3.11-suc docker.io/longhornio/support-bundle-kit:v0.0.51 docker.io/rancher/harvester-node-manager:v0.3.4 docker.io/rancher/mirrored-grafana-grafana:9.1.5 docker.io/rancher/fleet:v0.11.2 docker.io/rancher/harvester-load-balancer-webhook:v0.4.4 docker.io/rancher/mirrored-kiwigrid-k8s-sidecar:1.24.6 docker.io/longhornio/csi-attacher:v4.8.0 docker.io/rancher/harvester-network-helper:v0.5.6 docker.io/rancher/mirrored-prometheus-operator-prometheus-operator:v0.65.1 docker.io/rancher/shell:v0.1.26 docker.io/rancher/mirrored-kube-state-metrics-kube-state-metrics:v2.10.1 docker.io/rancher/nginx-ingress-controller:v1.12.1-hardened1 docker.io/rancher/rancher-agent:v2.10.1 docker.io/longhornio/backing-image-manager:v1.7.3 docker.io/longhornio/longhorn-manager:v1.7.3 docker.io/longhornio/longhorn-ui:v1.7.3 docker.io/rancher/fleet-agent:v0.11.2 docker.io/rancher/system-upgrade-controller:v0.14.2 ghcr.io/kube-logging/config-reloader:v0.0.5 registry.suse.com/suse/sles/15.6/virt-controller:1.3.1-150600.5.9.1 docker.io/rancher/harvester-networkfs-manager:v0.1.2 docker.io/rancher/harvester-pcidevices:v0.4.3 docker.io/rancher/harvester-webhook:v1.4.3 docker.io/rancher/rancher-webhook:v0.6.2 docker.io/longhornio/csi-snapshotter:v7.0.2-20250204 docker.io/rancher/hardened-dns-node-cache:1.24.0-build20241211 docker.io/rancher/harvester-eventrouter:v0.3.3 registry.suse.com/suse/sles/15.6/virt-launcher:1.3.1-150600.5.9.1 docker.io/rancher/harvester-node-manager-webhook:v0.3.4 docker.io/rancher/mirrored-kube-logging-logging-operator:4.4.0 docker.io/rancher/mirrored-prometheus-adapter-prometheus-adapter:v0.10.0 docker.io/rancher/kubectl:v1.20.2 docker.io/rancher/harvester-node-disk-manager:v0.7.11 docker.io/rancher/mirrored-ingress-nginx-kube-webhook-certgen:v20221220-controller-v1.5.1-58-g787ea74b6 docker.io/rancher/mirrored-prometheus-operator-prometheus-config-reloader:v0.65.1 docker.io/rancher/hardened-etcd:v3.5.19-k3s1-build20250306 docker.io/rancher/hardened-kubernetes:v1.31.7-rke2r1-build20250312 docker.io/rancher/hardened-multus-cni:v4.1.4-build20250108 registry.suse.com/suse/sles/15.6/libguestfs-tools:1.3.1-150600.5.9.1 registry.suse.com/suse/sles/15.6/virt-operator:1.3.1-150600.5.9.1 docker.io/rancher/hardened-cluster-autoscaler:v1.9.0-build20241126 docker.io/rancher/harvester-cluster-repo:v1.4.3 docker.io/rancher/harvester-network-webhook:v0.5.6 docker.io/rancher/harvester-vm-import-controller:v0.4.3 docker.io/rancher/shell:v0.1.24 registry.suse.com/suse/sles/15.6/virt-api:1.3.1-150600.5.9.1 docker.io/fluent/fluent-bit:2.1.8 docker.io/longhornio/csi-provisioner:v4.0.1-20250204 docker.io/rancher/harvester-load-balancer:v0.4.4 docker.io/rancher/mirrored-prometheus-node-exporter:v1.3.1 docker.io/longhornio/csi-resizer:v1.13.1 docker.io/rancher/rke2-cloud-provider:v1.31.2-0.20241016053446-0955fa330f90-build20241016 docker.io/longhornio/livenessprobe:v2.15.0 docker.io/rancher/rke2-runtime:v1.31.7-rke2r1 registry.suse.com/suse/sles/15.6/virt-handler:1.3.1-150600.5.9.1 docker.io/rancher/hardened-calico:v3.29.2-build20250306 docker.io/rancher/mirrored-cluster-api-controller:v1.8.3 docker.io/rancher/mirrored-prometheus-prometheus:v2.45.0 docker.io/rancher/rancher:v2.10.1 docker.io/rancher/harvester-seeder:v0.4.3 docker.io/rancher/mirrored-prometheus-alertmanager:v0.26.0 docker.io/rancher/system-agent-installer-rke2:v1.31.7-rke2r1 ghcr.io/kube-logging/fluentd:v1.15-ruby3 docker.io/rancher/klipper-helm:v0.9.4-build20250113 docker.io/longhornio/longhorn-share-manager:v1.7.3 docker.io/rancher/hardened-coredns:v1.12.0-build20241126]"#;
        let expected = "info";
        let actual = sb_search.find_log_level(line).unwrap();
        assert_eq!(actual, expected);

        let line = r#"2025-12-08T07:55:50.064883108Z time="2025-12-08T07:55:50Z" level=error msg="error syncing 'fleet-local/request-x49zj': handler cluster-registration: failed to delete fleet-local/request-x49zj rbac.authorization.k8s.io/v1, Kind=RoleBinding for cluster-registration fleet-local/request-x49zj: rolebindings.rbac.authorization.k8s.io \"request-x49zj\" not found, requeuing"#;
        let expected = "error";
        let actual = sb_search.find_log_level(line).unwrap();
        assert_eq!(actual, expected);

        let line = r#"2025-12-08T10:30:36.714032412Z time="2025-12-08T10:30:36Z" level=debug msg="Prepare to encode to yaml file path: /tmp/support-bundle-kit/bundle/yamls/namespaced/fleet-local/v1/configmaps.yaml"#;
        let expected = "debug";
        let actual = sb_search.find_log_level(line).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_find_log_level_pattern2() {
        let sb_search = SBSearch::new("test").unwrap();

        let line = r#"2025-12-08T07:31:53.675701835Z {"level":"warn","ts":"2025-12-08T07:31:53.675659Z","caller":"etcdserver/util.go:170","msg":"apply request took too long","took":"122.37201ms","expected-duration":"100ms","prefix":"read-only range ","request":"key:\"/registry/pods/cattle-fleet-local-system/fleet-agent-77c65c9d9d-pxttp\" limit:1 ","response":"range_response_count:0 size:7"}"#;
        let expected = "warn";
        let actual = sb_search.find_log_level(line).unwrap();
        assert_eq!(actual, expected);

        let line = r#"2025-12-08T07:31:53.675709316Z {"level":"info","ts":"2025-12-08T07:31:53.675686Z","caller":"traceutil/trace.go:171","msg":"trace[1928396386] range","detail":"{range_begin:/registry/pods/cattle-fleet-local-system/fleet-agent-77c65c9d9d-pxttp; range_end:; response_count:0; response_revision:89089900; }","duration":"122.440061ms","start":"2025-12-08T07:31:53.553239Z","end":"2025-12-08T07:31:53.675679Z","steps":["trace[1928396386] 'agreement among raft nodes before linearized reading'  (duration: 122.37561ms)"],"step_count":1}"#;
        let expected = "info";
        let actual = sb_search.find_log_level(line).unwrap();
        assert_eq!(actual, expected);

        let line = r#"2025-12-08T10:27:24.459805082Z {"level":"info","ts":"2025-12-08T10:27:24Z","logger":"bundle","msg":"Unchanged bundledeployment","controller":"bundle","controllerGroup":"fleet.cattle.io","controllerKind":"Bundle","Bundle":{"name":"mcc-rancher-monitoring-crd","namespace":"fleet-local"},"namespace":"fleet-local","name":"mcc-rancher-monitoring-crd","reconcileID":"60a1cd4d-9ddf-4248-a6c6-c1353dab3e71","manifestID":"s-f2fb94554dbed0b86084cd509f78763ed14e1338a52bd90ee7a4b7ff53e0a","bundledeployment":{"metadata":{"name":"mcc-rancher-monitoring-crd","namespace":"cluster-fleet-local-local-1a3d67d0a899","creationTimestamp":null,"labels":{"fleet.cattle.io/bundle-name":"mcc-rancher-monitoring-crd","fleet.cattle.io/bundle-namespace":"fleet-local","fleet.cattle.io/cluster":"local","fleet.cattle.io/cluster-namespace":"fleet-local","fleet.cattle.io/managed":"true"},"finalizers":["fleet.cattle.io/bundle-deployment-finalizer"]},"spec":{"paused":true,"stagedOptions":{"defaultNamespace":"cattle-monitoring-system","helm":{"releaseName":"rancher-monitoring-crd","version":"105.1.2+up61.3.2","timeoutSeconds":600},"ignore":{}},"stagedDeploymentID":"s-f2fb94554dbed0b86084cd509f78763ed14e1338a52bd90ee7a4b7ff53e0a:90a578a64e92227563052c8bf1f175c182d754a1955e3222f1b8f6dcdabb5ee8","options":{"defaultNamespace":"cattle-monitoring-system","helm":{"releaseName":"rancher-monitoring-crd","version":"105.1.2+up61.3.2","timeoutSeconds":600},"ignore":{}},"deploymentID":"s-f2fb94554dbed0b86084cd509f78763ed14e1338a52bd90ee7a4b7ff53e0a:90a578a64e92227563052c8bf1f175c182d754a1955e3222f1b8f6dcdabb5ee8"},"status":{"display":{},"resourceCounts":{"ready":0,"desiredReady":0,"waitApplied":0,"modified":0,"orphaned":0,"missing":0,"unknown":0,"notReady":0}}},"deploymentID":"s-f2fb94554dbed0b86084cd509f78763ed14e1338a52bd90ee7a4b7ff53e0a:90a578a64e92227563052c8bf1f175c182d754a1955e3222f1b8f6dcdabb5ee8","operation":"unchanged"}"#;
        let expected = "info";
        let actual = sb_search.find_log_level(line).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_find_log_level_pattern3() {
        let sb_search = SBSearch::new("test").unwrap();
        let line = r#"2025-12-08T07:27:14.834602400Z E1208 07:27:14.834539       1 job_controller.go:631] "Unhandled Error" err="syncing job: tracking status: adding uncounted pods to status: Operation cannot be fulfilled on jobs.batch \"fleet-cleanup-clusterregistrations\": the object has been modified; please apply your changes to the latest version and try again" logger="UnhandledError"
"#;
        let expected = "error";
        let actual = sb_search.find_log_level(line).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_find_log_level_pattern4() {
        let sb_search = SBSearch::new("test").unwrap();
        let line = r#"2025-12-08T07:47:45.565219601Z 2025/12/08 07:47:45 [error] 3099#3099: *7756 upstream prematurely closed connection while reading upstream, client: 192.168.48.101, server: rancher.192.168.48.100.example.org, request: "GET /apis/fleet.cattle.io/v1alpha1/namespaces/cluster-fleet-default-mgmt-bb69eaf374c2/bundledeployments?allowWatchBookmarks=true&resourceVersion=20055629&timeoutSeconds=479&watch=true HTTP/2.0", upstream: "http://10.52.0.2:80/apis/fleet.cattle.io/v1alpha1/namespaces/cluster-fleet-default-mgmt-bb69eaf374c2/bundledeployments?allowWatchBookmarks=true&resourceVersion=20055629&timeoutSeconds=479&watch=true", host: "rancher.192.168.48.100.example.org"
"#;
        let expected = "error";
        let actual = sb_search.find_log_level(line).unwrap();
        assert_eq!(actual, expected);

        let line = r#"2025-12-08T08:23:35.438311029Z 2025/12/08 08:23:35 [ERROR] error syncing 'fleet-local/local-managed-system-upgrade-controller': handler mcc-bundle: configmaps "" not found, requeuing"#;
        let expected = "error";
        let actual = sb_search.find_log_level(line).unwrap();
        assert_eq!(actual, expected);
    }
}
