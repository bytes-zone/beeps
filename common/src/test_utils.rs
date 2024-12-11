use crate::{hlc::Hlc, node_id::NodeId};
use chrono::{TimeZone, Utc};
use proptest::prop_compose;

prop_compose! {
    pub fn clock()
        (node_id: NodeId, timestamp in 0i64..2_000_000_000i64) -> Hlc {
        Hlc::new_at(
            node_id,
            Utc.timestamp_opt(timestamp, 0).unwrap(),
        )
    }
}
