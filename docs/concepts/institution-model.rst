The Institution Model
=====================

Why Babylon models institutions as a third entity layer between substrate
and agents, how the three-faction balance of forces works, why structural
selectivity matters for gameplay, and how institutions relate to
Althusser's theory of state apparatuses.

.. contents:: On this page
   :local:
   :depth: 2


Why a Third Layer?
------------------

The Babylon simulation originally had two entity layers: *substrate*
(SocialClass, Territory, Community) and *agents* (Organizations). This
two-layer model has a gap. Organizations come and go -- a police
department can be defunded, a union can be dissolved, a factory can
close. But the *function* that these organizations serve persists. A
new police department will be created to replace the old one. A new
factory will open. The function of policing, of employment, of
education -- these persist because the social relations that produce
them persist.

Institutions are the layer that captures this persistence. An institution
is not an organization. It is the crystallized social relation that
generates organizations. The Department of Justice is an institution; the
FBI is an organization housed within it. The Catholic Church is an
institution; a specific parish is an organization housed within it. Ford
Motor Company is an institution; its Detroit assembly plant is an
organization housed within it.

The distinction matters for gameplay. When the player attacks an
organization, they face tactical questions: can we disrupt its
leadership, cut its funding, reduce its legitimacy? When the player
confronts an institution, they face strategic questions: what social
function does this institution serve? Can we build alternative
institutions that serve the same function? What happens when we
destroy the organization but the institution persists and simply
spawns a replacement?

This three-layer architecture -- substrate, institution, agent --
models the historical observation that revolutionary movements fail
not because they cannot defeat individual state organizations, but
because they cannot replace the institutional functions those
organizations serve. The Bolsheviks dissolved the Tsarist police; they
had to immediately build a new police apparatus, because the function
of policing (however redefined) did not disappear with the organization
that had performed it.


Althusser's Apparatus Classification
-------------------------------------

Every institution carries an ``apparatus_type`` drawn from Althusser's
distinction between Repressive State Apparatuses (RSA) and Ideological
State Apparatuses (ISA), extended with a third Economic category.

**RSA types** -- ``RSA_EXECUTIVE``, ``RSA_MILITARY``, ``RSA_POLICE``,
``RSA_JUDICIAL``, ``RSA_CARCERAL`` -- operate primarily through
repression. They are the organs of state violence: courts, prisons,
police departments, the military. Their structural selectivity makes
repression cheap and education expensive. An RSA institution does not
need to convince anyone -- it coerces.

**ISA types** -- ``ISA_EDUCATIONAL``, ``ISA_RELIGIOUS``, ``ISA_FAMILY``,
``ISA_LEGAL``, ``ISA_POLITICAL``, ``ISA_COMMUNICATIONS``,
``ISA_CULTURAL`` -- operate primarily through ideology. Schools, churches,
media, the family -- these are the apparatuses that reproduce class
relations through consent rather than force. Their structural selectivity
makes education and recruitment cheap, repression expensive. An ISA
institution needs legitimacy to function.

**Economic types** -- ``ECONOMIC_PRODUCTIVE``, ``ECONOMIC_FINANCIAL``,
``ECONOMIC_EXTRACTIVE`` -- operate through surplus extraction. Factories,
banks, mining operations. Their structural selectivity makes employment
and fundraising cheap, reflecting the material basis of their influence.

The classification is not decorative. It determines the default action
cost modifiers that shape how organizations housed within an institution
can act. A revolutionary party housed within a university (ISA_EDUCATIONAL)
can recruit cheaply but cannot repress. A security agency housed within
the Department of Justice (RSA_JUDICIAL) can surveil cheaply but educating
the public costs double. The institution's structure selects for certain
kinds of action and against others.


Social Function and Persistence
-------------------------------

Every institution serves a ``social_function`` -- a material need of the
population. Employment, education, worship, policing, healthcare, care,
adjudication, communication. These functions are the reason institutions
persist through organizational turnover.

The key insight is that social functions are needs, not organizations.
The need for dispute resolution (``ADJUDICATION``) exists independently
of any particular court system. The need for meaning-making (``WORSHIP``)
exists independently of any particular church. When an organization that
serves a social function is destroyed, the institution creates a
replacement because the need still exists.

This is modeled through ``ReproductionMechanism`` and
``SpawningBlueprint``. An institution with a recruitment pipeline,
training program, succession protocol, budget independence, and legal
mandate has high ``reproduction_capacity`` -- it can efficiently replace
lost organizations. An institution lacking these mechanisms reproduces
slowly or not at all.

The reproduction capacity formula weights boolean mechanisms at 70% and
budget independence at 30%::

   reproduction_capacity = (sum(bools) / 4) * 0.7 + budget_independence * 0.3

An institution with all four mechanisms and full budget independence
scores 1.0 (maximum reproduction). An institution with no mechanisms
and no budget independence scores 0.0 (cannot reproduce).


The Three-Faction Balance
-------------------------

The most significant departure from treating institutions as static
structures is the ``InternalBalanceOfForces`` model. Within every
institution, three ruling-class fractions compete for hegemony:

**Liberal-Technocratic** faction seeks to maintain class rule through
consent -- assimilation, co-optation, procedural legitimacy. When
liberal-technocratic hegemony prevails, the institution favors
ASSIMILATE actions and has high escalation reluctance (0.7). This
faction weakens when legitimacy erodes.

**Revanchist-Fascist** faction seeks class rule through naked repression.
When revanchist-fascist hegemony prevails, the institution favors REPRESS
actions and has low escalation reluctance (0.2). This faction strengthens
during crisis.

**Institutionalist-Bonapartist** faction prioritizes the institution's own
survival and independence over the interests of any particular class
fraction. When bonapartist hegemony prevails, the institution favors
SURVEIL actions with moderate escalation reluctance (0.5). This faction
strengthens under external threat.

The balance shifts through alpha-smoothed dynamics:

- Rising crisis intensity drives the REVANCHIST weight up.
- Falling legitimacy weakens the LIBERAL weight.
- External threat drives the BONAPARTIST weight up.

Weights are renormalized after each update to maintain the sum-to-1.0
invariant. When the hegemonic fraction changes, a ``FactionShiftEvent``
is generated. When the BONAPARTIST faction exceeds its threshold while
both other factions are below the exclusion threshold, a
``BonapartistModeEvent`` fires.

The three-faction model captures the historical observation that state
institutions are not monolithic instruments of a single class. The
American judiciary, for example, contains liberal proceduralists,
law-and-order revanchists, and institutionalist judges who prioritize
the court's independence above political alignment. The balance between
these tendencies shifts in response to material conditions, and that
shift changes how the institution acts in the world.


Structural Selectivity
----------------------

Poulantzas argued that the capitalist state is not a neutral instrument
wielded by the ruling class, but a structure that *selects* for certain
kinds of action and against others. A police department does not need to
be told to repress -- its structure makes repression the default, cheapest
action available.

Feature 040 implements this through a two-level modifier system:

1. **Apparatus-type defaults**: Each ``ApparatusType`` has a default set
   of action cost modifiers. ``RSA_POLICE`` makes ``repress`` cost 0.6x
   (40% cheaper) and ``educate`` cost 2.0x (100% more expensive).

2. **Institution-level overrides**: A specific institution can override
   any apparatus-type default through ``action_modifiers``. A reformed
   police department might have ``{"repress": 1.5, "educate": 0.8}`` --
   repression is now more expensive, education cheaper.

The lookup order is: institution override > apparatus-type default > 1.0
(no modifier). This means structural selectivity can be changed through
institutional reform, but the defaults encode the structural bias.

For the OODA Loop System (Feature 032), structural selectivity modifies
the cost of actions available to organizations housed within an
institution. An organization that wants to ``educate`` within an
RSA_CARCERAL institution pays 2.5x the normal action cost -- the
structure of a prison makes education enormously difficult, regardless
of the intentions of the people within it.


Class Inscription
-----------------

Every institution carries a ``class_inscription``: BOURGEOIS, PROLETARIAN,
or CONTESTED. This is distinct from Organization ``class_character`` in
two ways.

First, class inscription is more resistant to change. An organization's
class character can shift quickly through leadership changes or political
realignment. An institution's class inscription changes only through
sustained class struggle, operating on the same alpha-smoothed timescale
as the factional balance.

Second, class inscription reflects the institution's *structural*
relationship to class, not the intentions of its current occupants. The
Detroit Police Department has ``class_inscription=BOURGEOIS`` not because
every police officer supports the bourgeoisie, but because the
structural function of policing -- the protection of property relations
-- serves bourgeois class interests regardless of individual officers'
politics.

CONTESTED inscription means the institution is a site of active class
struggle. A public university might be CONTESTED: it reproduces class
relations through credentialing (bourgeois function) but also provides
tools for critical analysis (proletarian potential). The contest is
structural, not merely political.


Relationship to Feature 031 and Feature 039
---------------------------------------------

Feature 040 sits between two existing systems:

**Feature 031** (Organization Base Model) defines the organizations that
institutions house. The ``housed_org_ids`` field on Institution and the
``HOUSES`` edge type create the structural link. The ``is_institution``
and ``institutional_persistence`` fields on Organization are now
deprecated -- their function is replaced by the Institution entity.

**Feature 039** (State Apparatus AI) defines the AI decision-making for
state organizations. The ``hegemonic_fraction_effect`` function bridges
Features 039 and 040: it takes the hegemonic fraction from an
institution's internal balance and returns OODA modifier hints that
influence how housed state organizations choose their actions.

The data flow is: Institution (Feature 040) determines structural context
-> Organization OODA profile (Feature 032) is modulated by institutional
selectivity and factional effects -> State AI (Feature 039) makes
decisions within those constraints.


See Also
--------

- :doc:`/reference/institutions` -- Complete API reference
- :doc:`/concepts/organization-model` -- Organization Model explanation
- :doc:`/concepts/state-apparatus-ai` -- State Apparatus AI explanation
- :doc:`/concepts/ooda-loop-system` -- OODA Loop System explanation
- :doc:`/concepts/george-jackson-model` -- Consciousness formation theory
