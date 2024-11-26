use chrono::{DateTime, Utc};
use uuid::Uuid;

/// A representation of stored metadata about a client.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Client {
    /// The latest version for this client (may be the nil version)
    pub latest_version_id: Uuid,
    /// Data about the latest snapshot for this client
    pub snapshot: Option<Snapshot>,
}

/// Metadata about a snapshot, not including the snapshot data itself.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Snapshot {
    /// ID of the version at which this snapshot was made
    pub version_id: Uuid,

    /// Timestamp at which this snapshot was set
    pub timestamp: DateTime<Utc>,

    /// Number of versions since this snapshot was made
    pub versions_since: u32,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Version {
    /// The uuid identifying this version.
    pub version_id: Uuid,
    /// The uuid identifying this version's parent.
    pub parent_version_id: Uuid,
    /// The data carried in this version.
    pub history_segment: Vec<u8>,
}

/// A transaction in the storage backend.
///
/// Transactions must be sequentially consistent. That is, the results of transactions performed
/// in storage must be as if each were executed sequentially in some order. In particular,
/// un-committed changes must not be read by another transaction.
///
/// Changes in a transaction that is dropped without calling `commit` must not appear in any other
/// transaction.
pub trait StorageTxn {
    /// Get information about the given client
    fn get_client(&mut self, client_id: Uuid) -> anyhow::Result<Option<Client>>;

    /// Create a new client with the given latest_version_id
    fn new_client(&mut self, client_id: Uuid, latest_version_id: Uuid) -> anyhow::Result<()>;

    /// Set the client's most recent snapshot.
    fn set_snapshot(
        &mut self,
        client_id: Uuid,
        snapshot: Snapshot,
        data: Vec<u8>,
    ) -> anyhow::Result<()>;

    /// Get the data for the most recent snapshot.  The version_id
    /// is used to verify that the snapshot is for the correct version.
    fn get_snapshot_data(
        &mut self,
        client_id: Uuid,
        version_id: Uuid,
    ) -> anyhow::Result<Option<Vec<u8>>>;

    /// Get a version, indexed by parent version id
    fn get_version_by_parent(
        &mut self,
        client_id: Uuid,
        parent_version_id: Uuid,
    ) -> anyhow::Result<Option<Version>>;

    /// Get a version, indexed by its own version id
    fn get_version(&mut self, client_id: Uuid, version_id: Uuid)
        -> anyhow::Result<Option<Version>>;

    /// Add a version (that must not already exist), and
    ///  - update latest_version_id
    ///  - increment snapshot.versions_since
    fn add_version(
        &mut self,
        client_id: Uuid,
        version_id: Uuid,
        parent_version_id: Uuid,
        history_segment: Vec<u8>,
    ) -> anyhow::Result<()>;

    /// Commit any changes made in the transaction.  It is an error to call this more than
    /// once.  It is safe to skip this call for read-only operations.
    fn commit(&mut self) -> anyhow::Result<()>;
}

/// A trait for objects able to act as storage.  Most of the interesting behavior is in the
/// [`crate::storage::StorageTxn`] trait.
pub trait Storage: Send + Sync {
    /// Begin a transaction
    fn txn(&self) -> anyhow::Result<Box<dyn StorageTxn + '_>>;
}
