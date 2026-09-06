//! PER-23 Slice 3 headless CLI parser (ADR249 R10): a hand-rolled parser
//! over `std::env::args_os()` — no `clap` dependency exists in this
//! workspace and Slice 3 adds none. Parsing is a pure function over owned
//! words so every refusal is unit-testable without a process spawn; only
//! `main` touches the real argument vector and the environment.

use std::ffi::OsString;
use std::fmt;

use babylon_persistence::CampaignId;
use uuid::Uuid;

/// The `--headless` flag: run exactly one dossier command against the
/// fog-safe reader and exit instead of opening the windowed viewer.
pub const HEADLESS_FLAG: &str = "--headless";
/// The canonical campaign identity for the connected window or headless read.
/// Falls back to [`CAMPAIGN_ENV`].
pub const CAMPAIGN_FLAG: &str = "--campaign";
/// Environment fallback for the campaign identity when `--campaign` is
/// absent.
pub const CAMPAIGN_ENV: &str = "BABYLON_CAMPAIGN_ID";
/// Retired demo option, recognized only to return an explicit refusal.
pub const STORY_FLAG: &str = "--story";

/// The closed set of command words the parser recognizes. Anything else
/// earns a Levenshtein did-you-mean over exactly this list — never a
/// silent guess, never a prefix abbreviation.
const COMMAND_WORDS: [&str; 7] = [
    "changelog",
    "dossier",
    "help",
    "search",
    "show",
    "status",
    "tick",
];

const TOP_LEVEL_HELP: &str = "\
babylon-client — the Babylon viewer and headless dossier CLI

usage:
  babylon-client --campaign <uuid>
  babylon-client --headless [--campaign <uuid>] <command>

commands:
  dossier show <geoid>
      use this command for the dossier card of one county: title, durable
      and verified ticks, archive freshness, content hash, atoms, and the
      resolved place names its links point at.
  dossier search <query>
      use this command for the known-page search hits of one free-text
      query across the acknowledged Archive.
  tick status
      use this command for the durable committed tick tail of the campaign:
      the resolve tick, layout version, and the committed content and
      envelope hashes.
  changelog <geoid>
      use this command for the supersession feed of one county: the
      consecutive atom pairs whose atom identity changed across ticks.
  help [topic]
      use this command for the help text of any command topic, recursively.

options:
  --headless    run one command against the fog-safe reader and exit.
  --campaign    canonical campaign UUID; falls back to BABYLON_CAMPAIGN_ID.
  Open the connected window with `mise run play`.
";

const DOSSIER_HELP: &str = "\
dossier — county dossier reads through the fog-safe Archive reader

usage:
  dossier show <geoid>
      use this command for the dossier card of one county.
  dossier search <query>
      use this command for the known-page search hits of one query.
";

const DOSSIER_SHOW_HELP: &str = "\
dossier show <geoid> — one county's dossier card

use this command for the decision-ready card: the county title, the
durable committed tick, the archive-verified tick, the freshness state
(archive-current, archive-pending, or no-committed-tick), the exact
content hash of the acknowledged page, the structured atom composition,
and the place names the county's link atoms resolve to. Emits one JSON
object on stdout.

example:
  babylon-client --headless --campaign 00000000-0000-0000-0000-000000000001 \\
      dossier show 26163
";

const DOSSIER_SEARCH_HELP: &str = "\
dossier search <query> — known-page search through the fog-safe reader

use this command for every acknowledged page whose search text matches
the query: one JSON object per hit on stdout, carrying the page identity,
title, verified tick, and atom count.

example:
  babylon-client --headless --campaign 00000000-0000-0000-0000-000000000001 \\
      dossier search Wayne
";

const TICK_HELP: &str = "\
tick — durable committed tick reads

usage:
  tick status
      use this command for the durable committed tick tail of the campaign.
";

const TICK_STATUS_HELP: &str = "\
tick status — the campaign's durable committed tick tail

use this command for the acknowledged resolve tick, the envelope layout
version, and the exact committed tick content and envelope hashes, read
through the fog-safe tick-status view only. Emits one JSON object on
stdout; the durable tick is null when no tick committed yet.

example:
  babylon-client --headless --campaign 00000000-0000-0000-0000-000000000001 \\
      tick status
";

const CHANGELOG_HELP: &str = "\
changelog <geoid> — one county's supersession feed

use this command for the atom history of one county as a feed of rows:
every consecutive pair of visible atoms whose atom identity changed, in
signal-key then tick order, plus the initial appearance of each signal.
One JSON object per row on stdout.

example:
  babylon-client --headless --campaign 00000000-0000-0000-0000-000000000001 \\
      changelog 26163
";

const HELP_HELP: &str = "\
help [topic] — recursive help over the command tree

use this command for the help text of any topic: `help`, `help dossier`,
`help dossier show`, `help dossier search`, `help tick`, `help tick
status`, `help changelog`, or `help help`.
";

/// One parsed headless dossier command (ADR249 R9-R11).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CliCommand {
    /// `dossier show <geoid>` — the county dossier card.
    DossierShow {
        /// Five-digit county GEOID.
        geoid: String,
    },
    /// `dossier search <query>` — known-page search hits.
    DossierSearch {
        /// Free-text query against acknowledged page search text.
        query: String,
    },
    /// `tick status` — the committed durable tick status.
    TickStatus,
    /// `changelog <geoid>` — the supersession feed for one county.
    Changelog {
        /// Five-digit county GEOID.
        geoid: String,
    },
}

/// One recursively addressable help topic. Carried as a closed enum rather
/// than text so the value that reaches `print!` is a static page selected
/// by [`render_help`] — no parse-channel data flows to stdout.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HelpTopic {
    /// `--headless help` — the top-level command roster.
    Root,
    /// `help dossier`.
    Dossier,
    /// `help dossier show`.
    DossierShow,
    /// `help dossier search`.
    DossierSearch,
    /// `help tick`.
    Tick,
    /// `help tick status`.
    TickStatus,
    /// `help changelog`.
    Changelog,
    /// `help help`.
    Help,
}

/// The parsed command line: one windowed viewer run, one headless dossier
/// command, or a help topic that `main` renders and exits 0.
#[derive(Clone, Debug)]
pub enum CliRequest {
    /// Open the observer window for one durable campaign.
    Windowed {
        /// The canonical campaign identity.
        campaign_id: CampaignId,
    },
    /// Run exactly one headless dossier command.
    Headless {
        /// The command to run.
        command: CliCommand,
        /// The canonical campaign identity.
        campaign_id: CampaignId,
    },
    /// A help topic; `main` renders it with [`render_help`].
    Help(HelpTopic),
}

/// One loud CLI refusal. Every message carries the `file:line` of the site
/// that raised it, so a bare `eprintln!` in `main` stays diagnosable.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CliError {
    site: &'static str,
    message: String,
}

impl CliError {
    fn at(site: &'static str, message: String) -> Self {
        Self { site, message }
    }

    /// Borrow the `file:line` site that raised the refusal.
    #[must_use]
    pub const fn site(&self) -> &'static str {
        self.site
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "babylon-client {}: {}", self.site, self.message)
    }
}

impl std::error::Error for CliError {}

/// Parse the full argument vector (already excluding the program name) into one
/// [`CliRequest`]. Refuses loudly — never guesses — on an unknown command
/// word, a malformed campaign identity, a missing value, or a malformed
/// county GEOID.
///
/// # Errors
/// Refuses any argument vector that is not exactly one windowed run, one
/// headless command, or one help topic.
pub fn parse(args: impl IntoIterator<Item = OsString>) -> Result<CliRequest, CliError> {
    let words = into_words(args)?;
    let mut headless = false;
    let mut campaign: Option<String> = None;
    let mut rest: Vec<String> = Vec::new();
    let mut iter = words.into_iter();
    while let Some(word) = iter.next() {
        match word.as_str() {
            HEADLESS_FLAG => headless = true,
            CAMPAIGN_FLAG => {
                campaign = Some(iter.next().ok_or_else(|| {
                    CliError::at(
                        concat!(file!(), ":", line!()),
                        format!("{CAMPAIGN_FLAG} requires a value"),
                    )
                })?);
            }
            "--" => {
                rest.extend(iter);
                break;
            }
            _ => rest.push(word),
        }
    }
    if !headless {
        if !rest.is_empty() {
            return Err(CliError::at(concat!(file!(), ":", line!()),
                format!("windowed mode observes one durable campaign; use {HEADLESS_FLAG} for commands; demo stories are conformance-only")));
        }
        return Ok(CliRequest::Windowed {
            campaign_id: resolve_campaign(campaign)?,
        });
    }
    if rest.iter().any(|word| word == STORY_FLAG) {
        return Err(CliError::at(
            concat!(file!(), ":", line!()),
            format!("{STORY_FLAG} is conformance-only; the client observes a durable campaign"),
        ));
    }
    if rest.first().is_some_and(|word| word == "help") {
        return Ok(CliRequest::Help(parse_help_topic(&rest[1..])?));
    }
    let campaign_id = resolve_campaign(campaign)?;
    let command = parse_command(&rest)?;
    Ok(CliRequest::Headless {
        command,
        campaign_id,
    })
}

fn into_words(args: impl IntoIterator<Item = OsString>) -> Result<Vec<String>, CliError> {
    args.into_iter()
        .map(|arg| {
            arg.into_string().map_err(|_| {
                CliError::at(
                    concat!(file!(), ":", line!()),
                    "arguments must be valid UTF-8".to_owned(),
                )
            })
        })
        .collect()
}

fn resolve_campaign(flag: Option<String>) -> Result<CampaignId, CliError> {
    let raw = match flag {
        Some(value) => value,
        None => std::env::var(CAMPAIGN_ENV).map_err(|_| {
            CliError::at(
                concat!(file!(), ":", line!()),
                format!("no campaign identity: pass {CAMPAIGN_FLAG} <uuid> or set {CAMPAIGN_ENV}"),
            )
        })?,
    };
    let uuid = Uuid::parse_str(&raw).map_err(|_| {
        CliError::at(
            concat!(file!(), ":", line!()),
            format!("campaign identity must be a canonical UUID, got '{raw}'"),
        )
    })?;
    if uuid.is_nil() || uuid.to_string() != raw {
        return Err(CliError::at(
            concat!(file!(), ":", line!()),
            "campaign identity must be a nonzero canonical UUID".into(),
        ));
    }
    Ok(CampaignId::from_uuid(uuid))
}

fn parse_command(words: &[String]) -> Result<CliCommand, CliError> {
    let Some((first, tail)) = words.split_first() else {
        return Err(CliError::at(
            concat!(file!(), ":", line!()),
            format!("no command given; try `{HEADLESS_FLAG} help`"),
        ));
    };
    match first.as_str() {
        "dossier" => parse_dossier(tail),
        "tick" => match tail {
            [status] if status == "status" => Ok(CliCommand::TickStatus),
            _ => Err(CliError::at(
                concat!(file!(), ":", line!()),
                format!("expected 'tick status', got '{}'", tail.join(" ")),
            )),
        },
        "changelog" => match tail {
            [geoid] => Ok(CliCommand::Changelog {
                geoid: county_geoid(geoid)?,
            }),
            _ => Err(CliError::at(
                concat!(file!(), ":", line!()),
                format!(
                    "changelog expects one county GEOID, got '{}'",
                    tail.join(" ")
                ),
            )),
        },
        other => Err(unknown_word(other, "command")),
    }
}

fn parse_dossier(words: &[String]) -> Result<CliCommand, CliError> {
    let Some((sub, tail)) = words.split_first() else {
        return Err(CliError::at(
            concat!(file!(), ":", line!()),
            "dossier expects 'show <geoid>' or 'search <query>'".to_owned(),
        ));
    };
    match (sub.as_str(), tail) {
        ("show", [geoid]) => Ok(CliCommand::DossierShow {
            geoid: county_geoid(geoid)?,
        }),
        ("show", _) => Err(CliError::at(
            concat!(file!(), ":", line!()),
            "dossier show expects one county GEOID".to_owned(),
        )),
        ("search", [query]) => Ok(CliCommand::DossierSearch {
            query: query.clone(),
        }),
        ("search", _) => Err(CliError::at(
            concat!(file!(), ":", line!()),
            "dossier search expects one query".to_owned(),
        )),
        (other, _) => Err(unknown_word(other, "dossier subcommand")),
    }
}

fn county_geoid(raw: &str) -> Result<String, CliError> {
    if raw.len() == 5 && raw.bytes().all(|byte| byte.is_ascii_digit()) {
        return Ok(raw.to_owned());
    }
    Err(CliError::at(
        concat!(file!(), ":", line!()),
        format!("county GEOID must be exactly five ASCII digits, got '{raw}'"),
    ))
}

fn unknown_word(word: &str, role: &str) -> CliError {
    let suggestions = COMMAND_WORDS
        .iter()
        .filter(|candidate| levenshtein(word, candidate) <= 2)
        .map(|candidate| format!("'{candidate}'"))
        .collect::<Vec<_>>();
    let hint = if suggestions.is_empty() {
        String::new()
    } else {
        format!("; did you mean {}?", suggestions.join(", "))
    };
    CliError::at(
        concat!(file!(), ":", line!()),
        format!("unknown {role} '{word}'{hint}"),
    )
}

/// Parse one help topic address (`help`, `help dossier`, `help dossier
/// show`, ...) into its closed variant.
///
/// # Errors
/// Refuses an unknown topic with a did-you-mean, like any other word.
pub fn parse_help_topic(words: &[String]) -> Result<HelpTopic, CliError> {
    match words {
        [] => Ok(HelpTopic::Root),
        [topic] => match topic.as_str() {
            "dossier" => Ok(HelpTopic::Dossier),
            "tick" => Ok(HelpTopic::Tick),
            "changelog" => Ok(HelpTopic::Changelog),
            "help" => Ok(HelpTopic::Help),
            other => Err(unknown_word(other, "help topic")),
        },
        [topic, sub] => match (topic.as_str(), sub.as_str()) {
            ("dossier", "show") => Ok(HelpTopic::DossierShow),
            ("dossier", "search") => Ok(HelpTopic::DossierSearch),
            ("tick", "status") => Ok(HelpTopic::TickStatus),
            (other, _) => Err(unknown_word(other, "help topic")),
        },
        _ => Err(CliError::at(
            concat!(file!(), ":", line!()),
            format!(
                "help topics are at most two words deep, got '{}'",
                words.join(" ")
            ),
        )),
    }
}

/// Render a help topic. The returned page is always one of the pinned
/// static texts, so nothing from the parse channel reaches stdout.
#[must_use]
pub const fn render_help(topic: HelpTopic) -> &'static str {
    match topic {
        HelpTopic::Root => TOP_LEVEL_HELP,
        HelpTopic::Dossier => DOSSIER_HELP,
        HelpTopic::DossierShow => DOSSIER_SHOW_HELP,
        HelpTopic::DossierSearch => DOSSIER_SEARCH_HELP,
        HelpTopic::Tick => TICK_HELP,
        HelpTopic::TickStatus => TICK_STATUS_HELP,
        HelpTopic::Changelog => CHANGELOG_HELP,
        HelpTopic::Help => HELP_HELP,
    }
}

/// Exact Levenshtein edit distance over Unicode scalar values. Only the
/// did-you-mean uses it, over the seven closed command words.
fn levenshtein(left: &str, right: &str) -> usize {
    let right_chars = right.chars().collect::<Vec<_>>();
    let mut previous = (0..=right_chars.len()).collect::<Vec<_>>();
    let mut current = vec![0; previous.len()];
    for (row, left_char) in left.chars().enumerate() {
        current[0] = row + 1;
        for (column, right_char) in right_chars.iter().enumerate() {
            let substitution = previous[column] + usize::from(left_char != *right_char);
            current[column + 1] = (previous[column + 1] + 1)
                .min(current[column] + 1)
                .min(substitution);
        }
        std::mem::swap(&mut previous, &mut current);
    }
    previous[right_chars.len()]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::ffi::OsStringExt;

    fn os(words: &[&str]) -> Vec<OsString> {
        words.iter().map(OsString::from).collect()
    }

    const CAMPAIGN: &str = "00000000-0000-0000-0000-000000000001";

    fn campaign() -> CampaignId {
        CampaignId::from_uuid(Uuid::parse_str(CAMPAIGN).expect("test campaign parses"))
    }

    #[test]
    fn campaign_flag_opens_the_durable_window() {
        let request = parse(os(&[CAMPAIGN_FLAG, CAMPAIGN])).expect("campaign admits");
        assert!(
            matches!(request, CliRequest::Windowed { campaign_id } if campaign_id == campaign()),
            "the window binds the declared campaign"
        );
    }

    #[test]
    fn story_flag_cannot_open_a_parallel_windowed_simulation() {
        assert!(parse(os(&["--story", "carceral"])).is_err());
    }

    #[test]
    fn unknown_story_refuses_with_a_site_prefixed_error() {
        let error = parse(os(&["--story", "nope"])).expect_err("unknown story refuses");
        let rendered = error.to_string();
        assert!(
            rendered.starts_with("babylon-client ") && rendered.contains("cli.rs:"),
            "errors carry the file:line site prefix, got {rendered}"
        );
    }

    #[test]
    fn headless_tick_status_parses_with_the_campaign_flag() {
        let request = parse(os(&[
            HEADLESS_FLAG,
            CAMPAIGN_FLAG,
            CAMPAIGN,
            "tick",
            "status",
        ]))
        .expect("tick status admits");
        assert!(
            matches!(request, CliRequest::Headless { command: CliCommand::TickStatus, campaign_id } if campaign_id == campaign()),
            "the campaign flag pins the campaign identity"
        );
    }

    #[test]
    fn campaign_flag_is_interleavable_with_the_command_words() {
        let request = parse(os(&[
            "tick",
            "status",
            HEADLESS_FLAG,
            CAMPAIGN_FLAG,
            CAMPAIGN,
        ]))
        .expect("flags may follow the command words");
        assert!(matches!(
            request,
            CliRequest::Headless {
                command: CliCommand::TickStatus,
                ..
            }
        ));
    }

    #[test]
    fn campaign_falls_back_to_the_environment() {
        let env = crate::test_support::EnvVarGuard::lock(CAMPAIGN_ENV);
        env.set(CAMPAIGN);
        let request = parse(os(&[HEADLESS_FLAG, "tick", "status"])).expect("env fallback admits");
        drop(env);
        assert!(
            matches!(request, CliRequest::Headless { campaign_id, .. } if campaign_id == campaign()),
            "BABYLON_CAMPAIGN_ID admits when the flag is absent"
        );
    }

    #[test]
    fn missing_campaign_refuses_loudly() {
        let env = crate::test_support::EnvVarGuard::lock(CAMPAIGN_ENV);
        env.remove();
        let error =
            parse(os(&[HEADLESS_FLAG, "tick", "status"])).expect_err("missing campaign refuses");
        assert!(
            error.to_string().contains("BABYLON_CAMPAIGN_ID"),
            "the refusal names both admission paths, got {error}"
        );
    }

    #[test]
    fn story_flag_in_headless_mode_refuses_as_conformance_only() {
        let error = parse(os(&[
            HEADLESS_FLAG,
            STORY_FLAG,
            "carceral",
            "tick",
            "status",
        ]))
        .expect_err("a story flag in headless mode refuses");
        let message = error.to_string();
        assert!(
            message.contains(STORY_FLAG) && message.contains("conformance-only"),
            "the refusal names the flag and its conformance-only scope, got {error}"
        );
    }

    #[test]
    fn malformed_campaign_refuses_loudly() {
        let error = parse(os(&[
            HEADLESS_FLAG,
            CAMPAIGN_FLAG,
            "not-a-uuid",
            "tick",
            "status",
        ]))
        .expect_err("malformed campaign refuses");
        assert!(
            error.to_string().contains("canonical UUID"),
            "the refusal names the canonical UUID discipline"
        );
    }

    #[test]
    fn positional_commands_require_headless() {
        let error = parse(os(&[CAMPAIGN_FLAG, CAMPAIGN, "tick", "status"]))
            .expect_err("commands without headless refuse");
        assert!(
            error.to_string().contains(HEADLESS_FLAG),
            "the refusal points at the headless flag"
        );
    }

    #[test]
    fn dossier_show_parses_and_validates_the_geoid() {
        let request = parse(os(&[
            HEADLESS_FLAG,
            CAMPAIGN_FLAG,
            CAMPAIGN,
            "dossier",
            "show",
            "26163",
        ]))
        .expect("dossier show admits");
        assert!(
            matches!(request, CliRequest::Headless { command: CliCommand::DossierShow { geoid }, .. } if geoid == "26163")
        );
        let error = parse(os(&[
            HEADLESS_FLAG,
            CAMPAIGN_FLAG,
            CAMPAIGN,
            "dossier",
            "show",
            "2616",
        ]))
        .expect_err("a four-digit geoid refuses");
        assert!(error.to_string().contains("five ASCII digits"));
    }

    #[test]
    fn dossier_search_parses_the_query_verbatim() {
        let request = parse(os(&[
            HEADLESS_FLAG,
            CAMPAIGN_FLAG,
            CAMPAIGN,
            "dossier",
            "search",
            "Wayne County",
        ]))
        .expect("dossier search admits");
        assert!(
            matches!(request, CliRequest::Headless { command: CliCommand::DossierSearch { query }, .. } if query == "Wayne County")
        );
    }

    #[test]
    fn changelog_parses_the_geoid() {
        let request = parse(os(&[
            HEADLESS_FLAG,
            CAMPAIGN_FLAG,
            CAMPAIGN,
            "changelog",
            "26163",
        ]))
        .expect("changelog admits");
        assert!(
            matches!(request, CliRequest::Headless { command: CliCommand::Changelog { geoid }, .. } if geoid == "26163")
        );
    }

    #[test]
    fn unknown_command_earns_a_did_you_mean() {
        let error = parse(os(&[
            HEADLESS_FLAG,
            CAMPAIGN_FLAG,
            CAMPAIGN,
            "dossiar",
            "show",
            "26163",
        ]))
        .expect_err("a near-miss command refuses");
        assert!(
            error.to_string().contains("did you mean 'dossier'"),
            "the suggestion names the closed word, got {error}"
        );
    }

    #[test]
    fn empty_headless_command_refuses_with_help_hint() {
        let error = parse(os(&[HEADLESS_FLAG, CAMPAIGN_FLAG, CAMPAIGN]))
            .expect_err("a missing command refuses");
        assert!(error.to_string().contains("help"));
    }

    #[test]
    fn help_topics_render_recursively_with_use_this_command_for() {
        for words in [
            vec![],
            vec!["dossier".to_owned()],
            vec!["dossier".to_owned(), "show".to_owned()],
            vec!["dossier".to_owned(), "search".to_owned()],
            vec!["tick".to_owned()],
            vec!["tick".to_owned(), "status".to_owned()],
            vec!["changelog".to_owned()],
            vec!["help".to_owned()],
        ] {
            let text = render_help(parse_help_topic(&words).expect("every topic parses"));
            assert!(
                text.contains("use this command for"),
                "topic '{words:?}' must say what it is for"
            );
        }
        let top = render_help(parse_help_topic(&[]).expect("top-level help parses"));
        for word in COMMAND_WORDS {
            assert!(top.contains(word), "top-level help lists '{word}'");
        }
    }

    #[test]
    fn help_route_parses_through_the_cli() {
        let request =
            parse(os(&[HEADLESS_FLAG, "help", "dossier", "show"])).expect("help route admits");
        let CliRequest::Help(topic) = request else {
            panic!("help routes to the Help request");
        };
        assert!(render_help(topic).contains("dossier show <geoid>"));
    }

    #[test]
    fn unknown_help_topic_earns_a_did_you_mean() {
        let error = parse_help_topic(&["tickk".to_owned()]).expect_err("a near-miss topic refuses");
        assert!(error.to_string().contains("did you mean 'tick'"));
    }

    #[test]
    fn levenshtein_counts_exact_edits() {
        assert_eq!(levenshtein("kitten", "sitting"), 3);
        assert_eq!(levenshtein("dossier", "dossier"), 0);
        assert_eq!(levenshtein("tick", "tickk"), 1);
        assert_eq!(levenshtein("", "status"), 6);
    }

    #[test]
    fn non_utf8_arguments_refuse_loudly() {
        let error = parse(vec![OsString::from_vec(vec![0xff])]).expect_err("non-UTF-8 refuses");
        assert!(error.to_string().contains("UTF-8"));
    }
}
