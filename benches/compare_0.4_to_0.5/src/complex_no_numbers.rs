use sscanf::FromScanf;

#[derive(Debug, FromScanf)]
#[sscanf("id={id}|name={name}|enabled={enabled}|retries={retries}|ratio={ratio}")]
pub(crate) struct ServiceConfig {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) enabled: bool,
    pub(crate) retries: String,
    pub(crate) ratio: String,
}

#[derive(Debug, FromScanf)]
#[sscanf(
    "host={host};port={port};secure={secure};region={region};zone={zone};timeout={timeout_ms}"
)]
pub(crate) struct NetworkEndpoint {
    pub(crate) host: String,
    pub(crate) port: String,
    pub(crate) secure: bool,
    pub(crate) region: String,
    pub(crate) zone: String,
    pub(crate) timeout_ms: String,
}

#[derive(Debug, FromScanf)]
#[sscanf("method={method},path={path},status={status},bytes={bytes},latency={latency_us}")]
pub(crate) struct RequestMetrics {
    pub(crate) method: String,
    pub(crate) path: String,
    pub(crate) status: String,
    pub(crate) bytes: String,
    pub(crate) latency_us: String,
}

#[derive(Debug, FromScanf)]
#[sscanf("config<{config}> endpoint<{endpoint}> metrics<{metrics}>")]
pub(crate) struct Deployment {
    pub(crate) config: ServiceConfig,
    pub(crate) endpoint: NetworkEndpoint,
    pub(crate) metrics: RequestMetrics,
}

#[derive(Debug, FromScanf)]
pub(crate) enum Payload {
    #[sscanf("primary<{deployment}>")]
    Primary { deployment: Deployment },
    #[sscanf("fallback<{endpoint}> reason={reason} code={code}")]
    Fallback {
        endpoint: NetworkEndpoint,
        reason: String,
        code: String,
    },
}

#[derive(Debug, FromScanf)]
#[sscanf(
    "trace={trace_id}|user={user_id}|source={source}|attempt={attempt}|payload<{payload}>|checksum={checksum}|finished={finished}"
)]
pub(crate) struct TelemetryRecord {
    pub(crate) trace_id: String,
    pub(crate) user_id: String,
    pub(crate) source: String,
    pub(crate) attempt: String,
    pub(crate) payload: Payload,
    pub(crate) checksum: String,
    pub(crate) finished: bool,
}

#[derive(Debug, FromScanf)]
#[sscanf(
    "BEGIN::version={version}::environment={environment}::record[{record}]::signature={signature}::END"
)]
pub(crate) struct BenchmarkMessage {
    pub(crate) version: String,
    pub(crate) environment: String,
    pub(crate) record: TelemetryRecord,
    pub(crate) signature: String,
}
