//! Asten's optional assistant.
//!
//! The assistant is an editor's helper, never a writer (D1, D27). It can run
//! the checks, explain a mark, look up the canon, search the notes, read the
//! timeline, summarise a source note and propose facts for the author to
//! settle. It cannot write, rewrite, continue or polish the author's prose,
//! settle canon, apply a repair, change settings, read a key or reach any host
//! but the provider the writer chose.
//!
//! Everything here is plain Rust with no window: the desktop shell wires the
//! commands, and the tests drive the same code against mock providers that
//! speak each provider's real wire format.
//!
//! The layers, outside in:
//!
//! 1. [`policy`]: the refusal path. A request to produce prose is refused
//!    before any provider is contacted; the system prompt says it again; a
//!    tool the model invents is refused; a reply that reads like new prose is
//!    withheld.
//! 2. [`config`], [`keys`], [`ledger`]: switches, consent, keys in the OS
//!    credential store (or an encrypted file), and the spend ledger with caps.
//! 3. [`provider`]: one adapter per wire format (Anthropic, OpenAI-style,
//!    Gemini), the HTTP transport and the SSE decoder.
//! 4. [`tools`]: the closed tool table over the local engine.
//! 5. [`runner`]: the loop that ties them together and reports events.

pub mod config;
pub mod error;
pub mod keys;
pub mod ledger;
pub mod policy;
pub mod provider;
pub mod runner;
pub mod secret;
pub mod tools;

pub use error::{AssistantError, Result};
pub use secret::Secret;
pub use tokio_util::sync::CancellationToken;
