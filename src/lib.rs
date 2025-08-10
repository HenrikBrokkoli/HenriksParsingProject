//! # Henrik's Parsing Project
//!
//! A library for building LL(1) parsers from a Backus-Naur-Form (BNF) like syntax.
//! These generated parsers can be coupled with a virtual machine to execute arbitrary code,
//! allowing you to build simple programming languages that can be executed.
//!
//! ## Features
//!
//! - Define custom language syntax using a BNF-like grammar
//! - Create a virtual machine (VM) to execute code written in your language
//! - Parse and execute programs in your custom language
//!
//! ## Example
//!
//! ```rust
//! use std::fs;
//! use henriks_parsing_project::script_parser::Parser;
//! use henriks_parsing_project::vms::NullVm;
//! use henriks_parsing_project::vms::VM;
//!
//! // Read the grammar rules
//! let rules = "start -> \"hello\" \"world\";";
//!
//! // Create a VM
//! let vm = NullVm::new();
//! let mut state = NullVm::create_new_state();
//!
//! // Create a parser
//! let mut parser = Parser::new_from_text(rules, &vm);
//!
//! // Parse a script
//! let script = "helloworld";
//! let result = parser.parse(script, &mut state);
//! ```
//!
//! ## Module Structure
//!
//! - `script_parser`: Core parser implementation
//! - `rule_parsing`: Parsing of grammar rules
//! - `vms`: Virtual machine implementations
//! - `parser_data`: Data structures for parser
//! - `first_sets`, `follow_sets`: LL(1) parsing algorithm components
//! - `errors`: Error types and handling

pub mod simple_graph;
pub mod named_graph;
pub mod rule_parsing;
pub mod script_parser;
pub mod first_sets;
pub mod sets;
pub mod test_helpers;
pub mod follow_sets;
pub mod steuer_sets;
pub mod steuer_map;
pub mod errors;
pub mod virtual_machine;
pub mod peekables;
pub mod parse_funcs;
pub mod vms;
pub mod parser_data;
mod tree;
