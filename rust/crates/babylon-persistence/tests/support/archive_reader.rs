//! Confined reader credentials for callers that already validated the disposable canary.

use super::{install_reader_role, ArchiveReadScope, CampaignId, SemanticArchiveReader};
use postgres::{Config, NoTls};

pub fn with_reader<T>(config: &Config, operation: impl FnOnce(&SemanticArchiveReader) -> T) -> T {
    install_reader_role(config).expect("install exact confined reader group");
    let role = format!(
        "per281_archive_reader_{}",
        format_args!(
            "{}_{:x}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock after epoch")
                .as_nanos()
        )
    );
    let mut admin = config.clone();
    admin.dbname("postgres");
    admin
        .connect(NoTls)
        .expect("reader role admin")
        .batch_execute(&format!(
        "CREATE ROLE {role} LOGIN PASSWORD 'archivereader' NOSUPERUSER NOCREATEDB NOCREATEROLE; \
        GRANT babylon_reader TO {role}; GRANT SET ON PARAMETER event_triggers TO {role}"
    ))
        .expect("create confined Archive login");
    let _cleanup = LoginCleanup {
        admin,
        role: role.clone(),
    };
    let mut confined = config.clone();
    confined.user(&role).password("archivereader");
    let reader = SemanticArchiveReader::new(&confined).expect("admit confined Archive reader");
    operation(&reader)
}

struct LoginCleanup {
    admin: Config,
    role: String,
}
impl Drop for LoginCleanup {
    fn drop(&mut self) {
        let result = self.admin.connect(NoTls).and_then(|mut client| {
            client.batch_execute(&format!(
                "REVOKE SET ON PARAMETER event_triggers FROM {role}; DROP ROLE {role}",
                role = self.role
            ))
        });
        if !std::thread::panicking() {
            result.expect("remove exact task-owned Archive login");
        }
    }
}

pub fn scope_at(config: &Config, campaign: CampaignId, tick: u64) -> ArchiveReadScope {
    let hash: Vec<u8> = config.connect(NoTls).expect("marker connection").query_one(
        "SELECT tick_content_hash FROM babylon_state.tick_commit WHERE campaign_id=$1 AND resolve_tick=$2",
        &[campaign.as_uuid(), &i64::try_from(tick).expect("bounded tick")]
    ).expect("exact committed marker").get(0);
    ArchiveReadScope::committed(campaign, tick, hash.try_into().expect("digest width"))
        .expect("exact read scope")
}
