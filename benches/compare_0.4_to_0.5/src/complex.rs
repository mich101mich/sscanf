use sscanf::FromScanf;

#[derive(Debug, FromScanf)]
#[sscanf("id={id}|name={name}|enabled={enabled}|retries={retries}|ratio={ratio}")]
pub(crate) struct ServiceConfig {
    pub(crate) id: u32,
    pub(crate) name: String,
    pub(crate) enabled: bool,
    pub(crate) retries: u8,
    pub(crate) ratio: f64,
}

#[derive(Debug, FromScanf)]
#[sscanf(
    "host={host};port={port};secure={secure};region={region};zone={zone};timeout={timeout_ms}"
)]
pub(crate) struct NetworkEndpoint {
    pub(crate) host: String,
    pub(crate) port: u16,
    pub(crate) secure: bool,
    pub(crate) region: String,
    pub(crate) zone: String,
    pub(crate) timeout_ms: u32,
}

#[derive(Debug, FromScanf)]
#[sscanf("method={method},path={path},status={status},bytes={bytes},latency={latency_us}")]
pub(crate) struct RequestMetrics {
    pub(crate) method: String,
    pub(crate) path: String,
    pub(crate) status: u16,
    pub(crate) bytes: u64,
    pub(crate) latency_us: u32,
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
        code: u16,
    },
}

#[derive(Debug, FromScanf)]
#[sscanf(
    "trace={trace_id}|user={user_id}|source={source}|attempt={attempt}|payload<{payload}>|checksum={checksum}|finished={finished}"
)]
pub(crate) struct TelemetryRecord {
    pub(crate) trace_id: String,
    pub(crate) user_id: u64,
    pub(crate) source: String,
    pub(crate) attempt: u8,
    pub(crate) payload: Payload,
    pub(crate) checksum: u64,
    pub(crate) finished: bool,
}

#[derive(Debug, FromScanf)]
#[sscanf(
    "BEGIN::version={version}::environment={environment}::record[{record}]::signature={signature}::END"
)]
pub(crate) struct BenchmarkMessage {
    pub(crate) version: u16,
    pub(crate) environment: String,
    pub(crate) record: TelemetryRecord,
    pub(crate) signature: String,
}
