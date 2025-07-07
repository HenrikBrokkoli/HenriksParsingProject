# Henrik's Parsing Project

This project provides tools to build LL(1) parsers from a Backus-Naur-Form (BNF) like syntax. These generated parsers can be coupled with a virtual machine to execute arbitrary code, allowing you to build simple programming languages that can be executed.

## Overview

Henrik's Parsing Project allows you to:
1. Define custom language syntax using a BNF-like grammar
2. Create a virtual machine (VM) to execute code written in your language
3. Parse and execute programs in your custom language

This is an ongoing project and should not be used for anything serious.

## How to Write Rule Files

Rule files define the grammar of your language using a BNF-like syntax. Here's how to write them:

### Basic Syntax

```
production_name -> expression;
```

Where:
- `production_name` is the name of a grammar production
- `expression` is a sequence of terminals and non-terminals

### Expressions

Expressions can contain:
- Terminal strings (in quotes): `"keyword"`
- Non-terminal references: `another_production`
- Alternatives (using `|`): `option1 | option2`
- Empty/epsilon production (using `#`): `optional_part -> something | #`
- VM actions (in curly braces): `number -> digit {}`

### Special Directives

- `$IGNORE: production_name;` - Defines productions to be ignored during parsing (like whitespace)

### Example Rule File

Here's a simple example for a stack-based calculator:

```
start  -> terms;

terms -> term terms_s;
terms_s -> whitespace term terms_s | #;

term -> add|sub|number|print;

print -> "print" {};
add -> "+" {};
sub -> "-" {};

number-> digit number_s {};
number_s -> number_s_ | # {};
number_s_ -> digit number_s {};
digit -> "0"|"1"|"2"|"3"|"4"|"5"|"6"|"7"|"8"|"9" {};

whitespaces -> whitespace whitespaces_s;
whitespaces_s -> whitespace whitespaces_s| #;
whitespace -> " ";
```

This grammar defines a language that can parse expressions like `1 2 + 3 + 4 - print`.

## How to Implement a VM

To implement a custom VM, you need to create a struct that implements the `VM` trait:

```rust
pub trait VM {
    type Tstate;
    type Tinstrution;

    fn parse_instructions<'a, T>(
        &'a self,
        prod_name: &str,
        to_parse: &mut ParseProcess<T>,
    ) -> Result<Vec<Self::Tinstrution>, ParserError>
    where
        T: TPeekable<Item = char>;

    fn execute_instruction(
        &self, 
        tree: &mut Tree<String>, 
        cur_node: NodeId, 
        instruction: &Self::Tinstrution, 
        state: &mut Self::Tstate
    );

    fn create_new_state() -> Self::Tstate;
}
```

### Key Components

1. **State Type (`Tstate`)**: Define a struct to hold your VM's state (e.g., stack, registers, memory)

2. **Instruction Type (`Tinstrution`)**: Define an enum for your VM's instructions

3. **parse_instructions**: Map grammar productions to VM instructions
   - Takes a production name and a parse process
   - Returns a vector of instructions

4. **execute_instruction**: Execute a specific instruction
   - Takes the parse tree, current node, instruction, and VM state
   - Modifies the VM state based on the instruction

5. **create_new_state**: Create a new VM state

### Example VM Implementation

Here's a simplified example of a stack-based VM:

```rust
pub struct SimpleStackVmState {
    pub stack: Vec<usize>,
    pub reg: usize,
    pub error: usize,
}

pub enum Instruction {
    Add,
    Sub,
    PushFromTree,
    PrintReg,
    // More instructions...
}

pub struct SimpleStackVm {}

impl VM for SimpleStackVm {
    type Tstate = SimpleStackVmState;
    type Tinstrution = Instruction;

    fn parse_instructions<T>(
        &self,
        prod_name: &str,
        to_parse: &mut ParseProcess<T>,
    ) -> Result<Vec<Self::Tinstrution>, ParserError>
    where
        T: TPeekable<Item = char>,
    {
        // Consume all characters in the parse process
        let mut c = to_parse.next();
        while let Some(_) = c {
            c = to_parse.next()
        }

        // Map production names to instructions
        let instruction = match prod_name {
            "add" => vec![Instruction::Add],
            "sub" => vec![Instruction::Sub],
            "print" => vec![Instruction::PrintReg],
            // More mappings...
            _ => vec![],
        };

        Ok(instruction)
    }

    fn execute_instruction(
        &self,
        tree: &mut Tree<String>,
        cur_node: NodeId,
        instruction: &Self::Tinstrution,
        state: &mut Self::Tstate,
    ) {
        match instruction {
            Instruction::Add => {
                // Pop two values, add them, push result
                // ...
            },
            // Handle other instructions...
        }
    }

    fn create_new_state() -> Self::Tstate {
        SimpleStackVmState { 
            stack: vec![],
            reg: 0,
            error: 0 
        }
    }
}
```

## How to Use the Project

### Basic Usage

1. **Define your grammar** in a rule file (e.g., `my_language.txt`)

2. **Implement a VM** for your language by creating a struct that implements the `VM` trait

3. **Create a parser** with your rules and VM:

```rust
use std::fs;
use HenriksParsingProject::script_parser::Parser;
use HenriksParsingProject::vms::VM;

// Read the grammar rules
let rules = fs::read_to_string("my_language.txt").expect("Unable to read rule file");

// Create your VM
let vm = MyCustomVm{};

// Create a VM state
let mut state = MyCustomVm::create_new_state();

// Create a parser with the rules and VM
let mut parser = Parser::new(&rules, &vm);

// Parse and execute a script
let script = "your script here";
let result = parser.parse(&script, &mut state);
```

### Example

Here's a complete example using the SimpleStackVm:

```rust
use std::fs;
use HenriksParsingProject::script_parser::Parser;
use HenriksParsingProject::vms::simple_stack_vm::SimpleStackVm;
use HenriksParsingProject::vms::VM;

fn main() {
    // Read the grammar rules
    let rules = fs::read_to_string("examples/simple_stack_based_math.txt")
        .expect("Unable to read rule file");

    // Create a SimpleStackVm
    let vm = SimpleStackVm{};

    // Create a VM state
    let mut state = SimpleStackVm::create_new_state();

    // Create a parser
    let mut parser = Parser::new(&rules, &vm);

    // Parse and execute a script
    // This script: puts 1 and 2 on stack, adds them, adds 3, subtracts 4, prints result
    let script = "1 2 + 3 + 4 - print";
    let _ = parser.parse(&script, &mut state).unwrap();

    // The result (2) should be printed to the console
}
```

## Examples

Check out the examples in the `examples` directory:
- [Simple Stack-Based Math](examples/simple_stack_based_math.rs): A simple calculator using a stack-based VM
- [Stack-Based Math](examples/stack_based_math.rs): A more complex calculator example

## Contributing

This is an ongoing project. Contributions are welcome!
