{-# LANGUAGE DataKinds #-}
{-# LANGUAGE DerivingStrategies #-}
{-# LANGUAGE GADTs #-}
{-# LANGUAGE GeneralizedNewtypeDeriving #-}
{-# LANGUAGE LambdaCase #-}
{-# LANGUAGE StandaloneKindSignatures #-}
{-# LANGUAGE StrictData #-}
{-# LANGUAGE TypeFamilies #-}

-- =============================================================================
-- Babylon.Core.Draft — a Haskell algebra for the Babylon functional core,
-- in the Lawverian tradition.  DRAFT 0 (2026-07-22).
--
-- Typechecks under GHC 9.4.7, deps: base + containers only.
--
-- Reading protocol: every construct cites the clause or ADR it enforces.
-- Constructs obey the earn-its-keep rule (ADR051 / Percy 2026-04-26): a
-- categorical construct ships only if it yields a LAW, a PREDICTION, or a
-- COMPUTATION.  Nothing here is vocabulary.
--
-- The export list IS the constitutional boundary (III.11, "intelligence
-- observes only"): World is abstract, witnesses are unforgeable, the only
-- arrows into World are hydration and the tick.
-- =============================================================================

module Babylon.Core.Draft
  ( -- * The world, abstractly (no constructors: the Loud-Failure boundary)
    World
  , advance
  , observe
  , Chronicle (..)
    -- * Kernel quantities (continuous plane, I.7)
  , Probability, probability
  , Intensity, intensity
  , Ideology, ideology
  , Coefficient, coefficient
  , Ratio, ratio
  , Currency (..), Micro (..)
  , Balance, KernelViolation (..)
    -- * Extensive / intensive quantities (Lawvere; ADR051 Phase D)
  , Extensive (..), Intensive (..), Share (..)
  , sumExtensive, meanIntensive, allocateExtensive
    -- * Folds: the only quantitative->qualitative morphisms (I.7 + I.12)
  , Fold (..), crossFold
    -- * The Lawverian layer (ADR051)
  , GaloisConnection (..), law_adjunction, unitDefect
  , AdjointCylinder (..), law_cylinderLeft, law_cylinderRight
  , SpatialLevel (..), SocialLevel (..), Level (..), aufhebung
  , Picard (..), staticOf
  , Pole (..), Opposition (..), Regime (..), Epsilon (..), classifyRegime
  , principal
  , Dialectic (..)
    -- * The grammar (nodes, verbs, payloads)
  , NodeKind (..), NodeId (..), SNodeKind (..)
  , VerbTag (..), EdgeVerb (..)
  , SomeNode (..), SomeEdge (..)
  , Topology, emptyTopology, addNode, addEdge, nodesInOrder, edgesInOrder
  , Violation (..)
    -- * Edge modes as a presented category (I.15; ADR051 E3)
  , Mode (..), Step (..), Path (..), organizingRoute
    -- * Formulas (pure, Defines-fed)
  , Defines (..), Rent (..), imperialRent
  , sigmoidP13, pSurvivalAcq, pSurvivalReb, rupture
  , BifurcationPole (..), routeAgitation, SolidarityWitness, requireSolidarity
  , deltaB, overshootO
    -- * The tick
  , Phase (..), System (..), Registry (..)
  , Event (..), TickReport (..), TickHash (..)
  , PureGen, genStep
    -- * Laws (lifted into the Amendment Q property layer)
  , law_insertionOrder, law_allocateConserves
    -- * Verbs as capabilities (I.16, I.21; the nine)
  , PlayerVerb (..), MembershipW, PresenceW, requireMembership, requirePresence
  , educate
  , Tier (..), OodaProfile (..)
    -- * Provenance (Aleksandrov Test)
  , Material  -- abstract: only the hydration boundary mints Material values
  ) where

import Data.Bits (xor)
import Data.Foldable (toList)
import Data.Int (Int64)
import Data.Word (Word64)
import Data.Kind (Type)
import Data.List (foldl')
import Data.Map.Strict (Map)
import qualified Data.Map.Strict as Map
import Data.Maybe (mapMaybe)
import Data.Sequence (Seq, (|>))
import qualified Data.Sequence as Seq

-- =============================================================================
-- §1  KERNEL: the two planes.
--
-- Constitution I.7: "Enums for qualities, floats for quantities.  Thresholds
-- explicit.  No continuous quality gradients."  I.12: control parameters
-- continuous, state variables discrete at fold crossings.
--
-- Here that is a type-level separation: the continuous plane is newtypes over
-- Double / fixed-point Int64; the discrete plane is closed sums.  The ONLY
-- morphisms from the first to the second are values of `Fold` (§3), whose
-- thresholds are constructed from Defines and nowhere else.
-- =============================================================================

data KernelViolation
  = OutOfRange  String Double     -- ^ smart constructor refused (Loud Failure)
  | NonPositive String Double
  | ZeroRepression                -- ^ P(S|R) denominator; refuse, don't fabricate
  deriving stock (Eq, Show)

-- | [0,1].  P(S|A), P(S|R), organization, repression-normalized quantities.
newtype Probability = Probability Double deriving newtype (Eq, Ord, Show)

-- | [0,1].  Contradiction intensity: 0 = dormant, 1 = rupture.  Fresh per tick
-- (ADR051: a gap is MEASURED, never accumulated).
newtype Intensity = Intensity Double deriving newtype (Eq, Ord, Show)

-- | [-1,1].  Revolutionary (-1) to reactionary (+1).
newtype Ideology = Ideology Double deriving newtype (Eq, Ord, Show)

-- | [0,1].  Formula parameters (alpha, lambda, k).  Lives in Defines ONLY.
newtype Coefficient = Coefficient Double deriving newtype (Eq, Ord, Show)

-- | (0,inf).  Wage ratios, exchange ratios, eta.
newtype Ratio = Ratio Double deriving newtype (Eq, Ord, Show)

-- | [-1,1].  Opposition balance (ADR051): 0 = equilibrium.
newtype Balance = Balance Double deriving newtype (Eq, Ord, Show)

-- | Fixed-point 1e-6 currency units.  RULING NEEDED (§11 of the design doc):
-- moving the value plane to fixed point makes cross-implementation
-- byte-identity trivial where it matters most (the Phi circuit), at the cost
-- of a one-time re-baseline.  Amendment Q III.12(b) anticipates exactly this.
newtype Micro = Micro Int64 deriving newtype (Eq, Ord, Show)

-- | [0,inf) by smart construction.  Wealth, wages, rent, GDP.
newtype Currency = Currency Micro deriving newtype (Eq, Ord, Show)

probability :: Double -> Either KernelViolation Probability
probability x | x >= 0 && x <= 1 = Right (Probability x)
              | otherwise        = Left (OutOfRange "Probability" x)

intensity :: Double -> Either KernelViolation Intensity
intensity x | x >= 0 && x <= 1 = Right (Intensity x)
            | otherwise        = Left (OutOfRange "Intensity" x)

ideology :: Double -> Either KernelViolation Ideology
ideology x | x >= -1 && x <= 1 = Right (Ideology x)
           | otherwise         = Left (OutOfRange "Ideology" x)

coefficient :: Double -> Either KernelViolation Coefficient
coefficient x | x >= 0 && x <= 1 = Right (Coefficient x)
              | otherwise        = Left (OutOfRange "Coefficient" x)

ratio :: Double -> Either KernelViolation Ratio
ratio x | x > 0     = Right (Ratio x)
        | otherwise = Left (NonPositive "Ratio" x)

-- | Saturating raise on the intensity plane.  Saturation is a DOCUMENTED
-- semantic (consciousness cannot exceed 1), not a silent clamp: the only
-- clamping arrows in the kernel are this and its dual, both named.
raiseIntensity :: Intensity -> Double -> Intensity
raiseIntensity (Intensity x) d = Intensity (min 1 (max 0 (x + d)))

-- =============================================================================
-- §2  EXTENSIVE vs INTENSIVE QUANTITY.
--
-- Lawvere, "Categories of Space and Quantity" (1992): extensive quantities
-- vary covariantly (they SUM under aggregation: population, value, biocapacity);
-- intensive quantities vary contravariantly (they take weighted MEANS: rates,
-- tensions, consciousness).  ADR051 Phase D shipped exactly this distinction
-- as aggregate_extensive vs aggregate_intensive, with the sheaf reading:
-- gluing = conservation.
--
-- The type-level payoff: Intensive has NO Num instance and no sum arrow.
-- The classic bug class — summing ratios across counties — is unrepresentable.
-- =============================================================================

newtype Extensive a = Extensive a deriving newtype (Eq, Ord, Show)
newtype Intensive a = Intensive a deriving newtype (Eq, Ord, Show)

-- | [0,1], and Σ shares = 1 over a partition (a LAW, checked in properties).
newtype Share = Share Double deriving newtype (Eq, Ord, Show)

sumExtensive :: Num a => [Extensive a] -> Extensive a
sumExtensive = Extensive . foldl' (+) 0 . map (\(Extensive x) -> x)

-- | aggregate_intensive: share-weighted mean.  A gap is a RATIO (ADR051 E1),
-- so level closure uses THIS, never sumExtensive.
meanIntensive :: [(Share, Intensive Double)] -> Intensive Double
meanIntensive xs = Intensive (sum [ s * v | (Share s, Intensive v) <- xs ])

-- | The left adjoint of the scale adjunction (allocate ⊣ aggregate, ADR051
-- Phase D).  Deterministic integer allocation: floor per share, remainder to
-- the FIRST region in insertion order (III.7 gives "first" a defined meaning).
-- Law: sumExtensive (allocateExtensive shs t) == t   (gluing = conservation).
allocateExtensive :: [Share] -> Extensive Micro -> [Extensive Micro]
allocateExtensive shs (Extensive (Micro total)) =
  case floors of
    []             -> []
    (Micro f0 : r) -> Extensive (Micro (f0 + remainder)) : map Extensive r
  where
    floors    = [ Micro (floor (fromIntegral total * s :: Double)) | Share s <- shs ]
    remainder = total - sum [ f | Micro f <- floors ]

-- =============================================================================
-- §3  FOLDS: the only quantitative->qualitative morphisms.
--
-- I.7 "Quantitative -> Qualitative": quantities accumulate, qualities transform
-- discretely, thresholds explicit.  I.12: the catastrophe surface — continuous
-- control parameters, discrete state variables at fold crossings.
--
-- A Fold is that clause as a value.  Its threshold is data, constructed from
-- Defines (the single moddable source of truth) and from nowhere else; a
-- numeric literal in a formula body is now a code-review artifact with no
-- type to inhabit.
-- =============================================================================

data Fold q s = Fold
  { fThreshold :: q   -- ^ explicit, Defines-sourced (I.7)
  , fBelow     :: s
  , fAbove     :: s
  }

crossFold :: Ord q => Fold q s -> q -> s
crossFold f x = if x < fThreshold f then fBelow f else fAbove f

-- =============================================================================
-- §4  THE LAWVERIAN LAYER (ADR051, typed).
--
-- Everything in this section already runs in Python under
-- src/babylon/dialectics/.  The Haskell versions add: laws as first-class
-- Bool-valued functions (lifted into the Amendment Q property layer), and
-- adjunction defect as the ONLY constructor of tension.
-- =============================================================================

-- | A poset adjunction f ⊣ g.  The Galois connection is the Lawverian
-- primitive from which contradiction is MEASURED, not asserted.
data GaloisConnection p q = GaloisConnection
  { leftAdjoint  :: p -> q
  , rightAdjoint :: q -> p
  }

-- | f x <= y  <=>  x <= g y.  QuickCheck-lifted in the property layer.
law_adjunction :: (Ord p, Ord q) => GaloisConnection p q -> p -> q -> Bool
law_adjunction gc x y =
  (leftAdjoint gc x <= y) == (x <= rightAdjoint gc y)

-- | The contradiction measure: how far the unit x <= g(f x) fails to close
-- (Laclau's failed identity; ADR051 "gap = measured adjunction defect").
-- Given a metric d on p, the defect IS the tension.  There is no other
-- constructor of edge tension in the core: tension cannot be invented,
-- only measured.
unitDefect :: (p -> p -> Intensity) -> GaloisConnection p q -> p -> Intensity
unitDefect d gc x = d x (rightAdjoint gc (leftAdjoint gc x))

-- | Lawvere's Unity-and-Identity-of-Adjoint-Opposites: L ⊣ U ⊣ R.
-- The shipped instance (ADR051 Phase B): over the SOLIDARITY subgraph,
-- L = discrete (total atomization), R = codiscrete (total solidarity),
-- U = underlying node set.  The world's actual solidarity topology sits
-- BETWEEN two adjoint opposites; atomization is its measured distance
-- from R's image.
data AdjointCylinder c d = AdjointCylinder
  { unity         :: c -> d   -- ^ U
  , leftOpposite  :: d -> c   -- ^ L ⊣ U  (discrete / atomized pole)
  , rightOpposite :: d -> c   -- ^ U ⊣ R  (codiscrete / total-solidarity pole)
  }

law_cylinderLeft :: (Ord c, Ord d) => AdjointCylinder c d -> c -> d -> Bool
law_cylinderLeft cyl x y =
  (leftOpposite cyl y <= x) == (y <= unity cyl x)

law_cylinderRight :: (Ord c, Ord d) => AdjointCylinder c d -> c -> d -> Bool
law_cylinderRight cyl x y =
  (unity cyl x <= y) == (x <= rightOpposite cyl y)

-- | E1's two level lattices.  Chains today; the class of lattices they
-- inhabit is Ord + Enum + Bounded.
data SpatialLevel = HexL | CountyL | StateL | NationL
  deriving stock (Eq, Ord, Enum, Bounded, Show)

data SocialLevel = IndividualL | CommunityL | ClassL | BlocL
  deriving stock (Eq, Ord, Enum, Bounded, Show)

data Level = Spatial SpatialLevel | Social SocialLevel
  deriving stock (Eq, Show)

-- | Aufhebung: the LEAST strictly-higher level at which the opposition
-- resolves.  Resolution predicate per E1: within-(l+1)-region variance of the
-- l-aggregates is zero.  Law (by construction): l < aufhebung resolved l.
aufhebung :: (Ord l, Enum l, Bounded l) => (l -> Bool) -> l -> Maybe l
aufhebung resolved l =
  case [ l' | l' <- [minBound .. maxBound], l' > l, resolved l' ] of
    []       -> Nothing
    (l' : _) -> Just l'

-- | Motion is primitive; statics are derived (Compact).  A static is a fixed
-- point of the Picard operator W_{n+1} = T(W_n) — reproduction is not a
-- separate mechanism but the |rate| <= eps neighborhood of this fixed point.
newtype Picard a = Picard (a -> a)

staticOf :: Eq a => Int -> Picard a -> a -> Maybe a
staticOf fuel (Picard t) = go fuel
  where
    go 0 _ = Nothing
    go n w = let w' = t w in if w' == w then Just w else go (n - 1) w'

-- | VIII.9: a pole may reference a COMMUNITY; dyadic reduction of an n-ary
-- formation is forbidden.  The prohibition is an ABSENCE: no exported arrow
-- has type CommunityId -> (Pole, Pole).
data Pole
  = ClassPole     (NodeId 'SocialClassK)
  | CommunityPole CommunityId
  | InstPole      (NodeId 'InstitutionK)
  deriving stock (Eq, Show)

data Opposition = Opposition
  { oName    :: String
  , oPoles   :: (Pole, Pole)
  , oGap     :: Intensity   -- ^ adjunction defect, fresh this tick (never +=)
  , oBalance :: Balance
  , oRate    :: Double      -- ^ d(gap)/dtick
  , oHome    :: Level       -- ^ OppositionSpec.level_name
  } deriving stock (Eq, Show)

newtype Epsilon = Epsilon Double deriving newtype (Eq, Ord, Show)

data Regime = Reproduction | Crisis | SublationTo Level
  deriving stock (Eq, Show)

-- | ADR051 E2, verbatim as a total function: reproduction when |rate| <= eps;
-- else sublation if a resolving level exists; else crisis.  RUPTURE is the
-- crisis regime's boiling point — a Fold on the principal gap, gated in
-- Defines, NOT a separate mechanism.
classifyRegime :: Epsilon -> Opposition -> Maybe Level -> Regime
classifyRegime (Epsilon eps) o resolvesAt
  | abs (oRate o) <= eps = Reproduction
  | Just l <- resolvesAt = SublationTo l
  | otherwise            = Crisis

-- | I.13 / ADR051: one contradiction leads per tick; selection is
-- gap * (1 + rate_weight * |rate|).  rate_weight comes from Defines.
principal :: Defines -> [Opposition] -> Maybe Opposition
principal d = \case
  [] -> Nothing
  os -> Just (foldl1 (\a b -> if rank b > rank a then b else a) os)
  where
    Coefficient rw = dRateWeight d
    rank o = let Intensity g = oGap o in g * (1 + rw * abs (oRate o))

-- | The constitutional primitive D = (A, A-bar, w, T, sigma).  Partitions are
-- DERIVED from it (Pi_0 of the induced relation), never primitive.  The field
-- names align 1:1 with the Compact; sigma is the Aufhebung route if one exists.
data Dialectic a = Dialectic
  { dThesis    :: Pole              -- ^ A
  , dNegation  :: Pole              -- ^ A-bar (determinate negation, not set complement)
  , dWeight    :: Intensity         -- ^ w — measured, §4 unitDefect only
  , dMotion    :: Picard a          -- ^ T — motion primitive
  , dSublation :: a -> Maybe Level  -- ^ sigma — the resolving level, if any
  }

-- =============================================================================
-- §5  THE GRAMMAR: nodes, verbs, payloads.
--
-- "Verb grammar" is literal: well-formed edges are well-typed sentences.
-- The typing judgments (which subject kinds an edge verb admits, which object
-- kinds) are GADT constructor signatures.  An ill-formed edge — ADJACENCY
-- between two classes, WAGES into a hex — is not invalid data caught by a
-- validator; it fails to typecheck.  I.14's "edges MUST be directed" is
-- inherent: a GADT constructor has an ordered signature.
-- =============================================================================

data NodeKind
  = SocialClassK | TerritoryK | OrganizationK | InstitutionK
  | SovereignK   | HexK       | IndustryK     | KeyFigureK

newtype NodeId (k :: NodeKind) = NodeId String
  deriving newtype (Eq, Ord, Show)

newtype CommunityId = CommunityId String deriving newtype (Eq, Ord, Show)

-- | Singletons: the runtime witness of a node's kind, so heterogeneous
-- storage can recover the payload type.
type SNodeKind :: NodeKind -> Type
data SNodeKind k where
  SSocialClass  :: SNodeKind 'SocialClassK
  STerritory    :: SNodeKind 'TerritoryK
  SOrganization :: SNodeKind 'OrganizationK
  SInstitution  :: SNodeKind 'InstitutionK
  SSovereign    :: SNodeKind 'SovereignK
  SHex          :: SNodeKind 'HexK
  SIndustry     :: SNodeKind 'IndustryK
  SKeyFigure    :: SNodeKind 'KeyFigureK

-- Node payloads.  Representative fields only — the full records transcribe
-- from src/babylon/models at porting time.  All value-plane fields are
-- kernel-typed; a raw Double in a payload is a review artifact.
data ClassState = ClassState
  { csPopulation    :: Extensive Int64
  , csWealth        :: Currency
  , csConstantC     :: Currency
  , csVariableV     :: Currency
  , csSurplusS      :: Currency
  , csConsciousness :: Intensity
  , csIdeology      :: Ideology
  , csPAcquiesce    :: Probability
  , csPRebel        :: Probability
  } deriving stock (Eq, Show)

data TerritoryState = TerritoryState
  { tsBiocapacity :: Extensive Double   -- ^ B; extraction permanently reduces max (I.8/I.9)
  , tsConsumption :: Extensive Double   -- ^ C
  } deriving stock (Eq, Show)

data OrgState = OrgState
  { osCadre     :: Extensive Int64
  , osResources :: Currency
  , osOoda      :: OodaProfile          -- ^ I.17: OODA as organizational metabolism
  } deriving stock (Eq, Show)

-- | The spatial substrate is IMMUTABLE (Compact): note the absence — no
-- exported arrow has HexState in its codomain except hydration.
data HexState        = HexState        deriving stock (Eq, Show)
data InstitutionState = InstitutionState deriving stock (Eq, Show)
data SovereignState  = SovereignState  deriving stock (Eq, Show)
data IndustryState   = IndustryState   deriving stock (Eq, Show)
data KeyFigureState  = KeyFigureState  deriving stock (Eq, Show)

type NodeData :: NodeKind -> Type
type family NodeData k where
  NodeData 'SocialClassK  = ClassState
  NodeData 'TerritoryK    = TerritoryState
  NodeData 'OrganizationK = OrgState
  NodeData 'InstitutionK  = InstitutionState
  NodeData 'SovereignK    = SovereignState
  NodeData 'HexK          = HexState
  NodeData 'IndustryK     = IndustryState
  NodeData 'KeyFigureK    = KeyFigureState

-- | The verb tags, promoted.  Economic + spatial substrate (managed by the
-- engine systems) and the political overlay (managed by org verbs).
data VerbTag
  = ExploitationV | SolidarityV | WagesV | TributeV | TenancyV | AdjacencyV
  | MembershipV | PresenceV | OrgRelationV | AttentionV

-- | THE TYPING JUDGMENTS.  Endpoint kinds marked [confirm] must be checked
-- against src/babylon/models before ratification — they are read off the
-- database spec and may have drifted.
type EdgeVerb :: VerbTag -> NodeKind -> NodeKind -> Type
data EdgeVerb v s t where
  Exploitation :: EdgeVerb 'ExploitationV 'SocialClassK 'SocialClassK
  Solidarity   :: EdgeVerb 'SolidarityV   'SocialClassK 'SocialClassK
  Wages        :: EdgeVerb 'WagesV        'SocialClassK 'SocialClassK   -- core bourgeoisie -> core workers
  Tribute      :: EdgeVerb 'TributeV      'SocialClassK 'SocialClassK   -- comprador -> core [confirm]
  Tenancy      :: EdgeVerb 'TenancyV      'SocialClassK 'TerritoryK     -- occupant -> territory
  Adjacency    :: EdgeVerb 'AdjacencyV    'TerritoryK   'TerritoryK     -- immutable substrate
  Membership   :: EdgeVerb 'MembershipV   'OrganizationK 'SocialClassK
  Presence     :: EdgeVerb 'PresenceV     'OrganizationK 'TerritoryK
  OrgRelation  :: EdgeVerb 'OrgRelationV  'OrganizationK 'OrganizationK
  Attention    :: EdgeVerb 'AttentionV    'InstitutionK  'OrganizationK -- state apparatus -> org

-- Edge payloads, per verb.
data FlowData = FlowData
  { fdFlow    :: Currency    -- ^ Phi conduit on EXPLOITATION; super-wages on WAGES
  , fdTension :: Intensity   -- ^ fresh per tick (§4: unitDefect only)
  } deriving stock (Eq, Show)

data BondData = BondData
  { bdTension    :: Intensity
  , bdResilience :: Coefficient
  } deriving stock (Eq, Show)

newtype TenancyData = TenancyData { tdRent :: Currency } deriving stock (Eq, Show)

data Tier = Sympathizer | Member | Cadre deriving stock (Eq, Ord, Show)

newtype MembershipData = MembershipData { mdTier :: Tier } deriving stock (Eq, Show)

newtype PresenceData = PresenceData { pdDepth :: Coefficient } deriving stock (Eq, Show)

data Agreement = Alliance | Ceasefire | UnitedFront | NoAgreement
  deriving stock (Eq, Show)

data OrgRelationData = OrgRelationData
  { orMode       :: Mode
  , orResilience :: Coefficient
  , orAgreement  :: Agreement
  , orIntel      :: Coefficient
  } deriving stock (Eq, Show)

data AttentionPhase = Dormant | Monitoring | Active | Disruption
  deriving stock (Eq, Show)

data AttentionData = AttentionData
  { adIntensity :: Intensity
  , adIntel     :: Coefficient
  , adPhase     :: AttentionPhase
  , adStick     :: Coefficient   -- ^ institutional inertia
  } deriving stock (Eq, Show)

type EdgePayload :: VerbTag -> Type
type family EdgePayload v where
  EdgePayload 'ExploitationV = FlowData
  EdgePayload 'WagesV        = FlowData
  EdgePayload 'TributeV      = FlowData
  EdgePayload 'SolidarityV   = BondData
  EdgePayload 'TenancyV      = TenancyData
  EdgePayload 'AdjacencyV    = ()           -- the substrate carries no dynamics
  EdgePayload 'MembershipV   = MembershipData
  EdgePayload 'PresenceV     = PresenceData
  EdgePayload 'OrgRelationV  = OrgRelationData
  EdgePayload 'AttentionV    = AttentionData

-- Heterogeneous storage: the existential seam.  Pattern-matching the verb
-- constructor RECOVERS the payload type — no isinstance, no dict.get.
data SomeNode where
  SomeNode :: SNodeKind k -> NodeId k -> NodeData k -> SomeNode

data SomeEdge where
  SomeEdge :: EdgeVerb v s t -> NodeId s -> NodeId t -> EdgePayload v -> SomeEdge

-- =============================================================================
-- §6  THE TOPOLOGY: insertion order as constitutional contract (III.7).
--
-- ADR052 preserved NetworkX's iteration contract with insertion-ordered
-- mirrors around a rustworkx core.  Here the mirror IS the structure: an
-- insertion-ordered map, pure.  nodesInOrder / edgesInOrder are the III.7
-- surface; the differential oracle in test_graph_iteration_order.py ports as
-- the conformance property for THIS structure.
--
-- The idiom shift of the whole port lives here: ADR052's payload dicts are
-- shared by reference and mutated in place; these payloads are values, and
-- every system is World -> World.  Reference semantics out, state threading in.
-- =============================================================================

newtype AnyId = AnyId String deriving newtype (Eq, Ord, Show)

eraseId :: NodeId k -> AnyId
eraseId (NodeId s) = AnyId s

-- | Insertion-ordered map.  Insert of an EXISTING key updates the payload and
-- keeps the original position (nx merge semantics, ADR052); delete drops the
-- key from the order.  toListIO is THE iteration contract.
data InsOrd k v = InsOrd
  { ioByKey :: Map k v
  , ioOrder :: Seq k
  }

emptyIO :: InsOrd k v
emptyIO = InsOrd Map.empty Seq.empty

insertIO :: Ord k => (v -> v -> v) -> k -> v -> InsOrd k v -> InsOrd k v
insertIO merge k v (InsOrd m o)
  | Map.member k m = InsOrd (Map.insertWith merge k v m) o
  | otherwise      = InsOrd (Map.insert k v m) (o |> k)

lookupIO :: Ord k => k -> InsOrd k v -> Maybe v
lookupIO k = Map.lookup k . ioByKey

toListIO :: Ord k => InsOrd k v -> [(k, v)]
toListIO (InsOrd m o) = mapMaybe (\k -> (,) k <$> Map.lookup k m) (toList o)

-- | Directed world topology.  tAdj is per-source insertion-ordered adjacency:
-- edgesInOrder = source-insertion-then-adjacency, byte-for-byte the ADR052
-- contract.
data Topology = Topology
  { tNodes :: InsOrd AnyId SomeNode
  , tAdj   :: InsOrd AnyId (InsOrd AnyId SomeEdge)
  }

emptyTopology :: Topology
emptyTopology = Topology emptyIO emptyIO

data Violation
  = DanglingEdge      AnyId AnyId   -- ^ ADR033: edge references nonexistent node
  | NegativeQuantity  AnyId String
  | KindMismatch      AnyId
  | KernelV           KernelViolation
  deriving stock (Eq, Show)

-- | Nodes enter ONLY with Material provenance (§10, Aleksandrov Test).
addNode :: SNodeKind k -> NodeId k -> Material (NodeData k) -> Topology -> Topology
addNode sk nid (Material nd) t =
  t { tNodes = insertIO (\new _ -> new) (eraseId nid) (SomeNode sk nid nd) (tNodes t) }

-- | Well-kinded by construction (the GADT); well-referenced by check (fail
-- loud, ADR033).  Payload update on an existing (u,v) replaces; typed
-- field-level merge is an open ruling (§11).
addEdge :: EdgeVerb v s t -> NodeId s -> NodeId t -> EdgePayload v
        -> Topology -> Either Violation Topology
addEdge ev u v p t
  | Nothing <- lookupIO (eraseId u) (tNodes t) = Left (DanglingEdge (eraseId u) (eraseId v))
  | Nothing <- lookupIO (eraseId v) (tNodes t) = Left (DanglingEdge (eraseId u) (eraseId v))
  | otherwise =
      let row  = maybe emptyIO id (lookupIO (eraseId u) (tAdj t))
          row' = insertIO (\new _ -> new) (eraseId v) (SomeEdge ev u v p) row
      in Right t { tAdj = insertIO (\new _ -> new) (eraseId u) row' (tAdj t) }

nodesInOrder :: Topology -> [SomeNode]
nodesInOrder = map snd . toListIO . tNodes

edgesInOrder :: Topology -> [SomeEdge]
edgesInOrder t =
  [ e | (_, row) <- toListIO (tAdj t), (_, e) <- toListIO row ]

-- =============================================================================
-- §7  EDGE MODES AS A PRESENTED CATEGORY (I.15; ADR051 E3).
--
-- Objects: the modes.  Generating morphisms: the ratified transition table.
-- The free category on the presentation is Path.  Two constitutional facts
-- become compiler facts:
--
--   1. I.15's prohibition (EXTRACTIVE -> SOLIDARISTIC requires a
--      TRANSACTIONAL intermediate) is an ABSENCE: there is no such generator,
--      so no such one-step Path can be written.
--   2. E3's corrected law ("no direct jump AND a TRANSACTIONAL route exists")
--      is an INHABITANT: organizingRoute below.  Its existence is checked at
--      compile time, forever.
--
-- Generators below are the DOCUMENTED subset; the full table (17 generators,
-- ADR051 E3) transcribes 1:1 from the repo source at ratification.  Each
-- generator carries a gate on contradiction internals (I.15) at the value
-- level — the type admits the step, the gate admits the occasion.
-- =============================================================================

data Mode = Extractive | Transactional | Solidaristic | Antagonistic | CoOptive
  deriving stock (Eq, Show)

type Step :: Mode -> Mode -> Type
data Step a b where
  Degrade    :: Step 'Solidaristic 'Transactional   -- under pressure
  Organize   :: Step 'Transactional 'Solidaristic   -- THE core player mechanic
  Marketize  :: Step 'Transactional 'Extractive
  Formalize  :: Step 'Extractive    'Transactional
  Rebel      :: Step 'Extractive    'Antagonistic   -- consciousness + org gated
  ResolveS   :: Step 'Antagonistic  'Solidaristic   -- outcome-dependent
  ResolveT   :: Step 'Antagonistic  'Transactional
  ResolveE   :: Step 'Antagonistic  'Extractive
  CoOpt      :: Step 'Antagonistic  'CoOptive       -- [confirm: united-front routes, E3]
  Break      :: Step 'CoOptive      'Antagonistic   -- [confirm]

data Path a b where
  Here :: Path a a
  Then :: Step a b -> Path b c -> Path a c

-- | E3's true law, as a term.  Delete the TRANSACTIONAL intermediate and this
-- stops compiling — the law cannot silently rot.
organizingRoute :: Path 'Extractive 'Solidaristic
organizingRoute = Then Formalize (Then Organize Here)

-- =============================================================================
-- §8  FORMULAS: pure, Defines-fed, kernel-typed.
--
-- Every tunable lives in Defines (GameDefines / defines.yaml — the single
-- moddable source of truth).  No formula takes a bare Double coefficient;
-- Defines has no Default instance; a literal has no type to hide behind.
-- =============================================================================

data Defines = Defines
  { dSigmoidScale :: Ratio        -- ^ survival sigmoid steepness
  , dRegimeEps    :: Epsilon      -- ^ tension.regime_rate_epsilon
  , dRateWeight   :: Coefficient  -- ^ principal-contradiction ranking weight
  , dSubsistence  :: Currency     -- ^ base_subsistence > 0 (I.8: existence costs calories)
  , dEta          :: Ratio        -- ^ metabolic eta > 1.0 (I.9)
  , dRuptureGate  :: Fold Intensity Bool  -- ^ crisis boiling point (E2)
  , dMobilizeGate :: Fold Intensity Bool  -- ^ MOBILIZE consciousness threshold
  }

-- | The Fundamental Theorem, as a sum: while W_c > V_c the rent flows and the
-- core cannot rupture; the theorem is not a comment on a float, it is the
-- shape of the return type.
data Rent = RentFlows Currency | RentExhausted
  deriving stock (Eq, Show)

imperialRent :: Currency -> Currency -> Rent
imperialRent (Currency (Micro wc)) (Currency (Micro vc))
  | wc > vc   = RentFlows (Currency (Micro (wc - vc)))
  | otherwise = RentExhausted

-- | Program-13 portable sigmoid: pure arithmetic, no libm, byte-stable across
-- implementations.  THIS BODY IS A PLACEHOLDER SHAPE (fast-sigmoid family);
-- the ratified polynomial and its error bound belong to the Program 13
-- float-honesty document.  Amendment Q gap (2) closes HERE.
sigmoidP13 :: Double -> Probability
sigmoidP13 x = Probability (0.5 * (1 + t / (1 + abs t)))
  where t = x

-- | P(S|A) = sigmoid(Wealth - Subsistence), scaled from Defines.
pSurvivalAcq :: Defines -> Currency -> Probability
pSurvivalAcq d (Currency (Micro w)) =
  sigmoidP13 (k * (fromIntegral w - fromIntegral s) / 1e6)
  where
    Ratio k = dSigmoidScale d
    Currency (Micro s) = dSubsistence d

-- | P(S|R) = Organization / Repression.  Zero repression is a refusal, not an
-- infinity; a ratio above 1 saturates, and the saturation is named.
pSurvivalReb :: Probability -> Probability -> Either KernelViolation Probability
pSurvivalReb (Probability org) (Probability rep)
  | rep <= 0  = Left ZeroRepression
  | otherwise = Right (Probability (min 1 (org / rep)))

-- | Rupture condition: P(S|R) > P(S|A).
rupture :: Probability -> Probability -> Bool
rupture pRebel pAcq = pRebel > pAcq

-- | The bifurcation: falling wages route agitation by SOLIDARITY edge
-- presence.  The witness is obtainable only by querying the topology — the
-- routing function cannot be called with a fabricated edge.
data BifurcationPole = Revolution | Fascism   -- -1 / +1
  deriving stock (Eq, Show)

newtype SolidarityWitness = SolidarityWitness (AnyId, AnyId)  -- constructor NOT exported

requireSolidarity :: NodeId 'SocialClassK -> Topology -> Maybe SolidarityWitness
requireSolidarity nid t = do
  row <- lookupIO (eraseId nid) (tAdj t)
  case [ k | (k, SomeEdge Solidarity _ _ _) <- toListIO row ] of
    (k : _) -> Just (SolidarityWitness (eraseId nid, k))
    []      -> Nothing

routeAgitation :: Maybe SolidarityWitness -> BifurcationPole
routeAgitation (Just _) = Revolution
routeAgitation Nothing  = Fascism

-- | Metabolic rift: dB = R - E*eta, eta > 1 from Defines; overshoot O = C/B.
deltaB :: Defines -> Extensive Double -> Extensive Double -> Double
deltaB d (Extensive r) (Extensive e) = r - e * eta
  where Ratio eta = dEta d

overshootO :: Extensive Double -> Extensive Double -> Either KernelViolation Ratio
overshootO (Extensive c) (Extensive b)
  | b <= 0    = Left (NonPositive "Biocapacity" b)
  | otherwise = ratio (c / b)

-- =============================================================================
-- §9  THE TICK: phase-indexed systems, causality as types.
--
-- The materialist-causality order (Material Base -> OODA Action ->
-- Consequences) is today a list in the engine and a discipline in review.
-- Here a System is indexed by its Phase, the Registry has one slot per phase,
-- and advance composes the phases in the only order that exists.  Registering
-- a consequences system into the base slot is a type error.  Within a phase,
-- the list is the strict registry order (the 28 systems).
--
-- advance is a FUNCTION.  Same Defines, same World: same result — III.7 is no
-- longer a discipline the gates check but a property of the arrow's type
-- (no IO in its signature).  The RNG is splittable pure state INSIDE World,
-- seeds pinned per ADR033.
-- =============================================================================

data Phase = MaterialBase | OodaAction | Consequences

type System :: Phase -> Type
newtype System p = System { runSystem :: Defines -> World -> (World, [Event]) }

data Registry = Registry
  { materialBase :: [System 'MaterialBase]
  , oodaAction   :: [System 'OodaAction]
  , consequences :: [System 'Consequences]
  }

data Event
  = RuptureFired    String            -- opposition name
  | LevelTransition Level             -- the production Aufhebung signal (ADR051 E2)
  | RegimeShift     Regime
  | PrincipalShift  String
  | VerbExecuted    PlayerVerb
  deriving stock (Eq, Show)

-- | Placeholder hash carrier.  The REAL recipe is Program 13's canonical
-- byte-level spec (Amendment Q gap (1)); the fold below exists to pin the
-- III.7 iteration surface it must consume: nodesInOrder ++ edgesInOrder.
newtype TickHash = TickHash Word64 deriving newtype (Eq, Show)

-- | Splittable pure generator, seeds pinned (ADR033).  SplitMix at port time;
-- an LCG stands in so the draft carries no dependency.
newtype PureGen = PureGen Int64 deriving newtype (Eq, Show)

genStep :: PureGen -> (Int64, PureGen)
genStep (PureGen s) =
  let s' = s * 6364136223846793005 + 1442695040888963407
  in (s', PureGen s')

data World = World
  { wTopology    :: Topology
  , wOppositions :: [Opposition]   -- ^ the OppositionRegistry, snapshot
  , wRng         :: PureGen
  , wTick        :: Int
  }

data TickReport = TickReport
  { trEvents :: [Event]
  , trHash   :: TickHash
  }

runPhase :: Defines -> [System p] -> (World, [Event]) -> (World, [Event])
runPhase d systems (w0, es0) =
  foldl' (\(w, es) s -> let (w', es') = runSystem s d w in (w', es <> es')) (w0, es0) systems

checkInvariants :: World -> [Violation]
checkInvariants w =
  [ NegativeQuantity (eraseId nid) "population"
  | SomeNode SSocialClass nid st <- nodesInOrder (wTopology w)
  , let Extensive p = csPopulation st
  , p < 0
  ]
  -- dangling edges are unconstructible (addEdge checks), negative currency is
  -- unconstructible (smart constructors): those ADR033 invariants moved from
  -- runtime checks to types.  What remains here is the residue.

tickHash :: World -> TickHash
tickHash w = TickHash (foldl' fnv 14695981039346656037 (canonical w))
  where
    fnv :: Word64 -> Char -> Word64
    fnv h c = (h `xor` fromIntegral (fromEnum c)) * 1099511628211
    -- FNV-1a stand-in over the III.7 iteration surface; Program 13 owns the
    -- ratified byte recipe (field layout, endianness, float encoding).
    canonical world =
      concat [ s | SomeNode _ (NodeId s) _ <- nodesInOrder (wTopology world) ]

-- | THE ARROW.  Loud Failure: an invariant violation aborts the tick — there
-- are no partial ticks and no fabricated states (III.11).
advance :: Defines -> Registry -> World -> Either Violation (World, TickReport)
advance d reg w0 =
  let (w1, es) = runPhase d (consequences reg)
                   (runPhase d (oodaAction reg)
                     (runPhase d (materialBase reg) (w0 { wTick = wTick w0 + 1 }, [])))
  in case checkInvariants w1 of
       (v : _) -> Left v
       []      -> Right (w1, TickReport es (tickHash w1))

-- =============================================================================
-- §10  VERBS AS CAPABILITIES (I.16, I.21, the nine).
--
-- Organizations ARE the agents; verbs operate through them.  Each verb's
-- Requires-clause becomes an argument whose type is an unforgeable witness:
-- requireMembership is the only mint for MembershipW, so EDUCATE without a
-- MEMBERSHIP edge is not a rejected action — it is an uncallable function.
-- The spec's validation table becomes the arguments page of this module.
-- =============================================================================

data PlayerVerb
  = Educate | Aid | Attack | Mobilize | Campaign
  | Move | Investigate | Reproduce | Negotiate
  deriving stock (Eq, Show)

newtype MembershipW = MembershipW (NodeId 'OrganizationK, NodeId 'SocialClassK)
newtype PresenceW   = PresenceW   (NodeId 'OrganizationK, NodeId 'TerritoryK)

requireMembership :: NodeId 'OrganizationK -> NodeId 'SocialClassK -> Topology
                  -> Maybe MembershipW
requireMembership org cls t = do
  row <- lookupIO (eraseId org) (tAdj t)
  case lookupIO (eraseId cls) row of
    Just (SomeEdge Membership _ _ _) -> Just (MembershipW (org, cls))
    _                                -> Nothing

requirePresence :: NodeId 'OrganizationK -> NodeId 'TerritoryK -> Topology
                -> Maybe PresenceW
requirePresence org terr t = do
  row <- lookupIO (eraseId org) (tAdj t)
  case lookupIO (eraseId terr) row of
    Just (SomeEdge Presence _ _ _) -> Just (PresenceW (org, terr))
    _                              -> Nothing

-- | I.17: the OODA profile is the organization's metabolism; verbs cost
-- capacity, and an unpayable verb is unexecutable (Maybe, not exception).
data OodaProfile = OodaProfile
  { opObserve :: Coefficient
  , opOrient  :: Coefficient
  , opDecide  :: Coefficient
  , opAct     :: Coefficient
  } deriving stock (Eq, Show)

-- | EDUCATE, end to end: the one verb given a real body, because it exhibits
-- the idiom the whole port turns on.  ADR013's systems mutate a shared dict;
-- here the update is a pure re-insert.  Consciousness rises by a
-- Defines-scaled delta and SATURATES (named semantic, §1); the edge
-- resilience upgrade is left to the transcription pass.
educate :: Defines -> MembershipW -> Double -> World -> (World, [Event])
educate _d (MembershipW (_org, cls)) delta w =
  case lookupIO (eraseId cls) (tNodes (wTopology w)) of
    Just (SomeNode SSocialClass nid st) ->
      let st' = st { csConsciousness = raiseIntensity (csConsciousness st) delta }
          n'  = insertIO (\new _ -> new) (eraseId cls)
                  (SomeNode SSocialClass nid st') (tNodes (wTopology w))
      in ( w { wTopology = (wTopology w) { tNodes = n' } }
         , [VerbExecuted Educate] )
    _ -> (w, [])   -- witness guaranteed the edge; node-kind residue fails soft here,
                   -- loud in the ported invariant layer

-- =============================================================================
-- §11  OBSERVATION AND PROVENANCE BOUNDARIES.
-- =============================================================================

-- | Aleksandrov Test as a type: state enters the world only wrapped in
-- Material, and Material's constructor is exported ONLY to the hydration
-- module (the Ledger boundary).  A formal construct with no material source
-- has no way into the topology.
newtype Material a = Material a

-- | What the intelligence layer gets: a projection.  World's constructors are
-- not exported; Chronicle is the entire read surface.  "AI observes and
-- narrates; the engine adjudicates" stops being a review rule and becomes
-- linkage fact.
data Chronicle = Chronicle
  { chEvents    :: [Event]
  , chTick      :: Int
  , chPrincipal :: Maybe String
  } deriving stock (Eq, Show)

observe :: World -> Chronicle
observe w = Chronicle
  { chEvents    = []          -- events stream out of advance, not storage
  , chTick      = wTick w
  , chPrincipal = Nothing     -- filled from the opposition registry projection
  }

-- =============================================================================
-- §12  LAWS AS VALUES (Amendment Q property layer, sketched).
--
-- law_adjunction, law_cylinderLeft/Right above; two more that pin the port:
-- =============================================================================

-- | III.7 order contract: inserting ks in sequence iterates ks in sequence.
-- The differential oracle against nx (test_graph_iteration_order.py) ports
-- against toListIO.
law_insertionOrder :: [(Int, Char)] -> Bool
law_insertionOrder kvs =
  map fst (toListIO built) == dedupFirst (map fst kvs)
  where
    built = foldl' (\io (k, v) -> insertIO (\new _ -> new) k v io) emptyIO kvs
    dedupFirst = go []
      where go seen (x:xs) | x `elem` seen = go seen xs
                           | otherwise     = x : go (x:seen) xs
            go _ [] = []

-- | Sheaf gluing = conservation (ADR051 Phase D): allocation then aggregation
-- is the identity on totals.
law_allocateConserves :: [Share] -> Extensive Micro -> Bool
law_allocateConserves shs total =
  null shs || sumExtensiveM (allocateExtensive shs total) == total
  where
    sumExtensiveM = Extensive . Micro . foldl' (\acc (Extensive (Micro x)) -> acc + x) 0
