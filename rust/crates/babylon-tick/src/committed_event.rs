//! Provenance-bearing event observations for committed ticks.
//!
//! Authored event payloads remain ordinary deterministic BSL values. The
//! engine adds the emitting rule and, only for an adjacent finite projection,
//! a receipt reference. Neither field is visible to mechanics.

use babylon_bsl::evaluator::Value;

use crate::choice_receipt::ChoiceReceiptRefV1;

/// One successful event plus engine-owned causal provenance.
#[derive(Debug, Clone, PartialEq)]
pub struct CommittedEventV2 {
    emitting_rule: String,
    choice_receipt: Option<ChoiceReceiptRefV1>,
    event_type: String,
    payload: Vec<(String, Value)>,
}

impl CommittedEventV2 {
    /// Own one event observation. The tick linker supplies receipt provenance;
    /// authored payload fields are never inspected to manufacture it.
    #[must_use]
    pub fn new(
        emitting_rule: String,
        choice_receipt: Option<ChoiceReceiptRefV1>,
        event_type: String,
        payload: Vec<(String, Value)>,
    ) -> Self {
        Self {
            emitting_rule,
            choice_receipt,
            event_type,
            payload,
        }
    }

    /// Borrow the rule that emitted this observation.
    #[must_use]
    pub fn emitting_rule(&self) -> &str {
        &self.emitting_rule
    }

    /// Return the adjacent finite-choice reference, when this is a projection.
    #[must_use]
    pub const fn choice_receipt(&self) -> Option<ChoiceReceiptRefV1> {
        self.choice_receipt
    }

    /// Borrow the canonical event type.
    #[must_use]
    pub fn event_type(&self) -> &str {
        &self.event_type
    }

    /// Borrow the authored observational payload.
    #[must_use]
    pub fn payload(&self) -> &[(String, Value)] {
        &self.payload
    }

    /// Convert to the legacy sink-shaped observation after durable metadata
    /// has already been retained in the tick report.
    #[must_use]
    pub fn sink_record(&self) -> (String, Vec<(String, Value)>) {
        (self.event_type.clone(), self.payload.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provenance_is_engine_owned_and_payload_remains_unchanged() {
        let event = CommittedEventV2::new(
            "struggle/spark-recognizer".to_owned(),
            Some(ChoiceReceiptRefV1::new(3)),
            "EXCESSIVE_FORCE".to_owned(),
            vec![("incident-tick".to_owned(), Value::Int(7))],
        );
        assert_eq!(event.emitting_rule(), "struggle/spark-recognizer");
        assert_eq!(
            event
                .choice_receipt()
                .map(ChoiceReceiptRefV1::encounter_ordinal),
            Some(3)
        );
        assert_eq!(
            event.sink_record(),
            (
                "EXCESSIVE_FORCE".to_owned(),
                vec![("incident-tick".to_owned(), Value::Int(7))]
            )
        );
    }
}
