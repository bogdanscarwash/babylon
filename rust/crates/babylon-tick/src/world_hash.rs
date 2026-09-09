//! Nominal whole-world hashing above the established graph-state hash.
//!
//! The graph hash remains its own stable contract. This layer adds only
//! auxiliary state that exists today: completed weekly time, both monotonic
//! graph allocator cursors, and the versioned static phase-schedule digest.
//! It does not invent future economic registers or persistence-envelope
//! fields.
//!
//! # Canonical byte layout — version 1
//!
//! Every integer is big-endian. The encoding is the fixed domain string
//! `babylon.world-state\0`, then `u32` layout version 1, followed by:
//!
//! ```text
//! 0x01 | 32-byte graph hash
//! 0x02 | i64 completed tick
//! 0x03 | u64 next-node cursor | u64 next-hyperedge cursor
//! 0x04 | 32-byte governed phase-schedule digest
//! ```

use babylon_graph::allocator_state::AllocatorCursors;
use babylon_kernel::sha256_of;

const WORLD_HASH_LAYOUT_VERSION: u32 = 1;
const WORLD_HASH_DOMAIN: &[u8] = b"babylon.world-state\0";

/// Hash one graph hash plus the real auxiliary state that names its world.
pub(crate) fn nominal_world_hash(
    graph_hash: [u8; 32],
    completed_tick: i64,
    cursors: AllocatorCursors,
    schedule_digest: [u8; 32],
) -> Result<[u8; 32], String> {
    Ok(sha256_of(&encode_nominal_world_state(
        graph_hash,
        completed_tick,
        cursors,
        schedule_digest,
    )?))
}

/// Encode the versioned nominal world identity before hashing it.
fn encode_nominal_world_state(
    graph_hash: [u8; 32],
    completed_tick: i64,
    cursors: AllocatorCursors,
    schedule_digest: [u8; 32],
) -> Result<Vec<u8>, String> {
    if completed_tick < 0 {
        return Err(format!(
            "completed tick must be non-negative, got {completed_tick}"
        ));
    }
    let mut bytes = Vec::with_capacity(116);
    bytes.extend_from_slice(WORLD_HASH_DOMAIN);
    bytes.extend_from_slice(&WORLD_HASH_LAYOUT_VERSION.to_be_bytes());
    bytes.push(0x01);
    bytes.extend_from_slice(&graph_hash);
    bytes.push(0x02);
    bytes.extend_from_slice(&completed_tick.to_be_bytes());
    bytes.push(0x03);
    bytes.extend_from_slice(&cursors.next_node.to_be_bytes());
    bytes.extend_from_slice(&cursors.next_hyperedge.to_be_bytes());
    bytes.push(0x04);
    bytes.extend_from_slice(&schedule_digest);
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::{encode_nominal_world_state, nominal_world_hash};
    use crate::phase_order::schedule_digest;
    use babylon_graph::allocator_state::{AllocatorCursors, AllocatorState};
    use babylon_graph::hypergraph_store::HypergraphStore;
    use babylon_graph::memory::MemoryGraph;
    use babylon_graph::state_hash::CanonicalState;
    use babylon_graph::substrate::GraphSubstrate;
    use babylon_kernel::sha256_of;

    fn seeded_world<G: GraphSubstrate + Default>(reverse_writes: bool) -> G {
        let mut graph = G::default();
        let worker = graph.add_node("SOCIAL_CLASS").unwrap();
        let organizer = graph.add_node("ORGANIZATION").unwrap();
        if reverse_writes {
            graph
                .update_node(organizer, "organization/cadre", 0.25)
                .unwrap();
            graph
                .update_node(worker, "social-class/wealth", 0.75)
                .unwrap();
            graph.add_hyperedge("CLASS", &[organizer, worker]).unwrap();
            graph
                .add_edge("SOLIDARITY", worker, organizer, 0.5)
                .unwrap();
        } else {
            graph
                .update_node(worker, "social-class/wealth", 0.75)
                .unwrap();
            graph
                .update_node(organizer, "organization/cadre", 0.25)
                .unwrap();
            graph
                .add_edge("SOLIDARITY", worker, organizer, 0.5)
                .unwrap();
            graph.add_hyperedge("CLASS", &[worker, organizer]).unwrap();
        }
        graph
    }

    fn seeded_world_hash<G: CanonicalState + AllocatorState>(graph: &G) -> [u8; 32] {
        nominal_world_hash(
            graph.state_hash().unwrap(),
            17,
            graph.allocator_cursors(),
            schedule_digest().unwrap(),
        )
        .unwrap()
    }

    #[test]
    fn empty_tick_zero_world_hash_pins_the_version_one_layout() {
        let graph = HypergraphStore::new();
        let hash = nominal_world_hash(
            graph.state_hash().unwrap(),
            0,
            graph.allocator_cursors(),
            schedule_digest().unwrap(),
        )
        .unwrap();
        assert_eq!(
            crate::hex(&hash),
            "bb1fd62f9053807b13fe00a45fc7f2c032dc9d23a4d778e8c5c016bbb59c2932"
        );
    }

    #[test]
    fn asymmetric_world_state_vector_pins_every_canonical_byte_and_its_sha() {
        let graph_hash = core::array::from_fn(|index| u8::try_from(index + 1).unwrap());
        let schedule_digest = core::array::from_fn(|index| u8::try_from(index).unwrap() + 0xa1);
        let cursors = AllocatorCursors {
            next_node: 0x1112_1314_1516_1718,
            next_hyperedge: 0x2122_2324_2526_2728,
        };
        let bytes =
            encode_nominal_world_state(graph_hash, 0x0102_0304_0506_0708, cursors, schedule_digest)
                .unwrap();

        #[rustfmt::skip]
        let expected: &[u8] = &[
            // Domain + u32 layout version 1.
            b'b', b'a', b'b', b'y', b'l', b'o', b'n', b'.',
            b'w', b'o', b'r', b'l', b'd', b'-', b's', b't', b'a', b't', b'e', 0x00,
            0x00, 0x00, 0x00, 0x01,
            // Section 0x01: asymmetric 32-byte graph hash.
            0x01,
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
            0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10,
            0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18,
            0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f, 0x20,
            // Section 0x02: signed completed tick, big-endian.
            0x02, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
            // Section 0x03: node cursor followed by hyperedge cursor.
            0x03,
            0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18,
            0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28,
            // Section 0x04: asymmetric 32-byte governed-schedule digest.
            0x04,
            0xa1, 0xa2, 0xa3, 0xa4, 0xa5, 0xa6, 0xa7, 0xa8,
            0xa9, 0xaa, 0xab, 0xac, 0xad, 0xae, 0xaf, 0xb0,
            0xb1, 0xb2, 0xb3, 0xb4, 0xb5, 0xb6, 0xb7, 0xb8,
            0xb9, 0xba, 0xbb, 0xbc, 0xbd, 0xbe, 0xbf, 0xc0,
        ];

        assert_eq!(bytes, expected);
        assert_eq!(
            crate::hex(&sha256_of(&bytes)),
            "8ce3e668779b762d26bf3543820775d52d711ffae6076006297c85968fb12751"
        );
    }

    #[test]
    fn allocator_history_moves_world_hash_when_live_graph_facts_match() {
        let untouched = HypergraphStore::new();
        let mut allocated = HypergraphStore::new();
        let transient = allocated.add_node("SOCIAL_CLASS").unwrap();
        allocated.remove_node(transient).unwrap();
        assert_eq!(
            untouched.state_hash().unwrap(),
            allocated.state_hash().unwrap()
        );

        let schedule = schedule_digest().unwrap();
        let a = nominal_world_hash(
            untouched.state_hash().unwrap(),
            0,
            untouched.allocator_cursors(),
            schedule,
        )
        .unwrap();
        let b = nominal_world_hash(
            allocated.state_hash().unwrap(),
            0,
            allocated.allocator_cursors(),
            schedule,
        )
        .unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn insertion_order_and_graph_store_do_not_move_nominal_world_identity() {
        let memory = seeded_world::<MemoryGraph>(false);
        let hypergraph = seeded_world::<HypergraphStore>(true);

        assert_eq!(
            memory.state_hash().unwrap(),
            hypergraph.state_hash().unwrap()
        );
        assert_eq!(memory.allocator_cursors(), hypergraph.allocator_cursors());
        assert_eq!(seeded_world_hash(&memory), seeded_world_hash(&hypergraph));
    }

    #[test]
    fn negative_completed_time_refuses_loudly() {
        let error = nominal_world_hash(
            [0; 32],
            -1,
            AllocatorCursors {
                next_node: 0,
                next_hyperedge: 0,
            },
            [0; 32],
        )
        .unwrap_err();
        assert_eq!(error, "completed tick must be non-negative, got -1");
    }
}
