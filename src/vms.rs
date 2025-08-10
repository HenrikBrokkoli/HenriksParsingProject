//! VM trait and reference implementations.
//!
//! This module defines the VM trait and provides several example VMs (e.g.,
//! stack-based variants) to use with the parser.
use crate::errors::ParserError;
use crate::peekables::{ParseProcess, TPeekable};
use crate::tree::{NodeId, Tree};

pub mod counting_vm;
pub mod simple_stack_vm;
pub mod stack_vm;

/// The Virtual Machine (VM) trait defines the interface for executing instructions
/// generated from parsing grammar productions.
///
/// Implementing this trait allows you to create a custom virtual machine that can
/// execute code written in your language. The VM is responsible for:
///
/// 1. Mapping grammar productions to instructions
/// 2. Executing those instructions
/// 3. Maintaining state during execution
///
/// # Example Implementation
///
/// ```rust
/// use henriks_parsing_project::vms::VM;
/// use henriks_parsing_project::tree::{Tree, NodeId};
/// use henriks_parsing_project::errors::ParserError;
/// use henriks_parsing_project::peekables::{ParseProcess, TPeekable};
///
/// // Define a simple VM for a calculator
/// pub struct CalculatorVm {}
///
/// // Define the VM state (e.g., a stack for calculations)
/// pub struct CalculatorState {
///     stack: Vec<i32>,
/// }
///
/// // Define instructions for the VM
/// pub enum Instruction {
///     Add,
///     Subtract,
///     PushNumber(i32),
/// }
///
/// impl VM for CalculatorVm {
///     type Tstate = CalculatorState;
///     type Tinstrution = Instruction;
///
///     // Map grammar productions to instructions
///     fn parse_instructions<'a, T>(
///         &'a self,
///         prod_name: &str,
///         to_parse: &mut ParseProcess<T>,
///     ) -> Result<Vec<Self::Tinstrution>, ParserError>
///     where
///         T: TPeekable<Item = char>
///     {
///         // Implementation details...
///         Ok(vec![])
///     }
///
///     // Execute an instruction
///     fn execute_instruction(
///         &self,
///         tree: &mut Tree<String>,
///         cur_node: NodeId,
///         instruction: &Self::Tinstrution,
///         state: &mut Self::Tstate
///     ) {
///         // Implementation details...
///     }
///
///     // Create a new VM state
///     fn create_new_state() -> Self::Tstate {
///         CalculatorState { stack: Vec::new() }
///     }
/// }
/// ```
pub trait VM {
    /// The type representing the VM's state.
    ///
    /// This is a separate type from the VM itself, allowing the state to be
    /// passed around and modified independently. The state typically contains
    /// data structures like stacks, registers, or memory that the VM operates on.
    type Tstate;

    /// The type representing instructions that the VM can execute.
    ///
    /// This is typically an enum with variants for different operations
    /// that your language supports (e.g., arithmetic operations, variable
    /// assignments, function calls).
    type Tinstrution;

    /// Maps grammar productions to VM instructions.
    ///
    /// This method is called during parsing when a production rule is matched.
    /// It serves two key purposes:
    ///
    /// 1. **Rule-based instructions**: Generate instructions based on the production name.
    ///    For example, when a rule named "add" is matched, generate an Add instruction.
    ///
    /// 2. **Custom code in curly braces**: Parse any code inside curly braces `{}` in the grammar rule.
    ///    For example, in a rule like `number -> digit { push_value() }`, parse and execute `push_value()`.
    ///
    /// # Arguments
    ///
    /// * `prod_name` - The name of the grammar production that was matched (e.g., "add", "number")
    /// * `to_parse` - The parse process containing any text inside curly braces `{}` from the grammar rule
    ///
    /// # Returns
    ///
    /// A vector of instructions to be executed later, or a ParserError if
    /// instruction generation fails.
    ///
    /// # Note
    ///
    /// The parse process must be fully consumed (by calling `next()` until it
    /// returns None) even if you don't use its contents.
    ///
    /// # Example
    ///
    /// For a grammar rule like:
    /// ```
    /// add -> "+" { push_add_instruction() };
    /// ```
    ///
    /// The `parse_instructions` method would:
    /// 1. Receive "add" as the `prod_name`
    /// 2. Receive " push_add_instruction() " in the `to_parse` parameter
    /// 3. Either use the production name "add" to determine the instruction
    /// 4. Or parse the content " push_add_instruction() " to determine the instruction
    fn parse_instructions<'a, T>(
        &'a self,
        prod_name: &str,
        to_parse: &mut ParseProcess<T>,
    ) -> Result<Vec<Self::Tinstrution>, ParserError>
    where
        T: TPeekable<Item = char>;

    /// Executes a single instruction, modifying the VM state.
    ///
    /// This method is called during execution for each instruction generated
    /// during parsing. It should interpret the instruction and modify the
    /// VM state accordingly.
    ///
    /// # Arguments
    ///
    /// * `tree` - The parse tree, which can be used to access parsed data
    /// * `cur_node` - The current node in the parse tree
    /// * `instruction` - The instruction to execute
    /// * `state` - The VM state to modify
    fn execute_instruction(&self, tree: &mut Tree<String>, cur_node: NodeId, instruction: &Self::Tinstrution, state: &mut Self::Tstate);

    /// Creates a new VM state.
    ///
    /// This method is called to initialize a fresh VM state before parsing
    /// and execution begin.
    ///
    /// # Returns
    ///
    /// A new instance of the VM state.
    fn create_new_state() -> Self::Tstate;
}

/// A no-operation virtual machine implementation.
///
/// This VM does nothing and is useful for:
/// - Testing the parser without executing any code
/// - Parsing text when you only need the parse tree and not execution
/// - Serving as a simple example of a VM implementation
#[derive(Debug)]
pub struct NullVm {}

impl NullVm {
    /// Creates a new NullVm instance.
    ///
    /// # Returns
    ///
    /// A new NullVm instance.
    pub fn new() -> NullVm {
        NullVm {}
    }

    /// A no-operation instruction function.
    ///
    /// This function does nothing and always returns Ok.
    ///
    /// # Arguments
    ///
    /// * `_tree` - Ignored parse tree
    ///
    /// # Returns
    ///
    /// Always returns Ok(())
    pub fn null_instruction(_tree: &mut Tree<String>) -> Result<(), String> {
        Ok(())
    }
}

impl VM for NullVm {
    /// The state type for NullVm is a simple usize that is never used.
    type Tstate = usize;

    /// The instruction type for NullVm is a simple usize that is never used.
    type Tinstrution = usize;

    /// Parses instructions for the NullVm.
    ///
    /// This implementation consumes all characters from the parse process
    /// but doesn't do anything with them. It always returns a vector with
    /// a single instruction (0).
    ///
    /// # Arguments
    ///
    /// * `_prod_name` - Ignored production name
    /// * `to_parse` - Parse process to consume
    ///
    /// # Returns
    ///
    /// Always returns Ok(vec![0])
    fn parse_instructions<'a, T>(
        &'a self,
        _prod_name: &str,
        to_parse: &mut ParseProcess<T>,
    ) -> Result<Vec<Self::Tinstrution>, ParserError>
    where
        T: TPeekable<Item = char>,
    {
        // Consume all characters in the parse process
        let mut c = to_parse.next();
        while let Some(_cc) = c {
            c = to_parse.next()
        }
        Ok(vec![0])
    }

    /// Executes an instruction for the NullVm.
    ///
    /// This implementation does nothing.
    ///
    /// # Arguments
    ///
    /// * `_` - Ignored parse tree
    /// * `_` - Ignored node ID
    /// * `_` - Ignored instruction
    /// * `_` - Ignored state
    fn execute_instruction(&self, _: &mut Tree<String>, _: NodeId, _: &Self::Tinstrution, _: &mut usize) {
        // Do nothing
    }

    /// Creates a new state for the NullVm.
    ///
    /// # Returns
    ///
    /// Always returns 0
    fn create_new_state() -> Self::Tstate {
        0
    }
}
