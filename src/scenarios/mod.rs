//! Module contenant le catalogue des attaques simulées et l'exécuteur de scénarios.
//!
//! Ce module regroupe :
//! - `catalogue` : définition et liste descriptive des 12 scénarios de sécurité.
//! - `executor` : exécution de simulations mockées/sécurisées via tokio.

pub mod catalogue;
pub mod executor;
