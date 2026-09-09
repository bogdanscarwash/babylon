; Organization practice circuit. A seeded organization spends one bounded
; action on rooted work. PRESENCE situates that work; material fields
; on ADJACENCY relations relay capacity; MEMBERSHIP supplies a finite social base;
; and COMMAND carries the state response. No territory label grants a bonus,
; no rule applies a sigmoid, and organization alone cannot fabricate care
; without stock, labor, and routing. This pack does not replenish action budget
; or write membership; those effects require the future committed-intent and
; attributed-membership boundaries.
;
; `organization/practice` and `organization/practice-embedding` are seeded
; scenario state in this slice. They prove the causal circuit but are not yet
; a player-input boundary; next-week intent can replace those seeds without
; changing the material rules.
;
(rule organization/p0-territory-inbox-reset
  :role mechanic
  :evidence derived
  :material-basis "Rooted work and territorial relay are this tick's contributions, so every territory clears both inboxes before organizations act."
  :fuel 32
  (bindings
    (binding rooted-work :field territory/rooted-work-inbox))
  (effects
    (update-node self territory/rooted-work-inbox (set 0.0c))
    (update-node self territory/rooted-relay-inbox (set 0.0c))))

(rule organization/p1-presence-attribution
  :role mechanic
  :evidence derived
  :material-basis "Each PRESENCE relation records two derived facts from the sealed snapshot: local-base population exists only when a declared MEMBERSHIP class has TENANCY in the same territory, and a practice branch exists only when the relation's material embedding matches the organization's chosen practice embedding. Recomputing both facts each tick prevents remote authorization and prevents a mismatched branch from consuming organizer labor."
  :fuel 512
  (domain NodeType/ORGANIZATION)
  (bindings
    (binding practice-embedding :field organization/practice-embedding))
  (when (exists (neighbors self EdgeType/PRESENCE :out NodeType/TERRITORY)))
  (effects
    (for-each (neighbors self EdgeType/PRESENCE :out NodeType/TERRITORY)
      :as base-territory
      (update-edge (edge-between EdgeType/PRESENCE self base-territory)
        presence/local-base-population
        (set
          (if (and (> (field-of base-territory territory/resident-population) 0)
                   (exists (neighbors self EdgeType/MEMBERSHIP :out NodeType/SOCIAL_CLASS)
                     :as base-class
                     (exists (neighbors base-class EdgeType/TENANCY :out NodeType/TERRITORY)
                       (= it base-territory))))
              (field-of base-territory territory/resident-population)
              0)))
      (update-edge (edge-between EdgeType/PRESENCE self base-territory)
        presence/practice-branch
        (set
          (if (and (= (field-of
                        (edge-between EdgeType/PRESENCE self base-territory)
                        presence/embedding)
                      practice-embedding)
                   (> (field-of base-territory territory/resident-population) 0)
                   (exists (neighbors self EdgeType/MEMBERSHIP :out NodeType/SOCIAL_CLASS)
                     :as branch-class
                     (exists (neighbors branch-class EdgeType/TENANCY :out NodeType/TERRITORY)
                       (= it base-territory))))
              1
              0))))))

(rule organization/p1-rooted-work
  :role mechanic
  :evidence derived
  :material-basis "Situated practice changes only territories reached through PRESENCE relations whose material embedding matches the organization's chosen practice. One available action is divided across those matching branches, so an organization cannot create free organizer labor by adding PRESENCE edges. Reproductive pressure conditions each branch's local gain, while circulation remains on the relations that carry it elsewhere."
  :fuel 512
  (bindings
    (binding active :field organization/active)
    (binding practice :field organization/practice)
    (binding practice-embedding :field organization/practice-embedding)
    (binding budget :field organization/action-budget)
    (binding practice-rate :const organization/practice-rate)
    (binding branch-count :expr
      (fold sum (neighbors self EdgeType/PRESENCE :out NodeType/TERRITORY)
        :as labor-branch
        (field-of
          (edge-between EdgeType/PRESENCE self labor-branch)
          presence/practice-branch))))
  (when (and (= active 1)
             (= practice PracticeKind/ROOTED_WORK)
             (> budget 0)
             (> branch-count 0)))
  (effects
    (for-each (neighbors self EdgeType/PRESENCE :out NodeType/TERRITORY)
      (guard (= (field-of
                  (edge-between EdgeType/PRESENCE self it)
                  presence/practice-branch)
                1)
        (update-node it territory/rooted-work-inbox
          (add (* (/ practice-rate branch-count)
                  (field-of it territory/reproduction-pressure))))))
    (update-node self organization/action-budget (sub 1))))

(rule organization/p2-territorial-relay
  :role mechanic
  :evidence derived
  :material-basis "Fresh rooted work becomes consequential across a declared circulation relation in proportion to its throughput and reproductive dependence, and in inverse proportion to alternate-route capacity and inventory buffers. Durable capacity does not copy itself into adjacent territories. No territory category or visible blockage grants intrinsic leverage."
  :fuel 256
  (bindings
    (binding rooted-work :field territory/rooted-work-inbox)
    (binding relay-rate :const organization/relay-rate))
  (when (and (> rooted-work 0.0c)
             (exists (neighbors self EdgeType/ADJACENCY :out NodeType/TERRITORY))))
  (effects
    (for-each (neighbors self EdgeType/ADJACENCY :out NodeType/TERRITORY)
      (update-node it territory/rooted-relay-inbox
        (add
          (*
            (*
              (*
                (*
                  (* rooted-work relay-rate)
                  (field-of
                    (edge-between EdgeType/ADJACENCY self it)
                    adjacency/throughput-share))
                (- 1.0c
                   (field-of
                     (edge-between EdgeType/ADJACENCY self it)
                     adjacency/alternate-capacity-share)))
              (- 1.0c
                 (field-of
                   (edge-between EdgeType/ADJACENCY self it)
                   adjacency/inventory-buffer-share)))
            (field-of
              (edge-between EdgeType/ADJACENCY self it)
              adjacency/reproduction-dependence)))))))

(rule organization/p3-rooted-capacity-apply
  :role mechanic
  :evidence derived
  :material-basis "A territory's durable rooted capacity is the bounded accumulation of prior capacity, situated work performed there, and fresh work relayed through declared adjacency."
  :fuel 32
  (bindings
    (binding capacity :field territory/rooted-capacity)
    (binding rooted-work :field territory/rooted-work-inbox)
    (binding rooted-relay :field territory/rooted-relay-inbox))
  (when (> (+ rooted-work rooted-relay) 0.0c))
  (effects
    (update-node self territory/rooted-capacity
      (set
        (if (< (+ (+ capacity rooted-work) rooted-relay) 1.0c)
            (+ (+ capacity rooted-work) rooted-relay)
            1.0c)))))

(rule organization/p5-command-response
  :role mechanic
  :evidence derived
  :material-basis "A state apparatus can answer rooted capacity only in territories reached through its declared COMMAND relation; the response grows against its remaining uncommitted capacity."
  :fuel 128
  (bindings
    (binding active :field organization/active)
    (binding kind :field organization/kind)
    (binding response-rate :const organization/command-response-rate))
  (when (and (= active 1)
             (= kind OrgKind/STATE_APPARATUS)
             (exists (neighbors self EdgeType/COMMAND :out NodeType/TERRITORY))))
  (effects
    (for-each (neighbors self EdgeType/COMMAND :out NodeType/TERRITORY)
      (update-node it territory/command-pressure
        (set
          (if (< (+ (field-of it territory/command-pressure)
                    (* (* response-rate
                          (field-of it territory/rooted-capacity))
                       (- 1.0c (field-of it territory/command-pressure))))
                 1.0c)
              (+ (field-of it territory/command-pressure)
                 (* (* response-rate
                       (field-of it territory/rooted-capacity))
                    (- 1.0c (field-of it territory/command-pressure))))
              1.0c))))))
